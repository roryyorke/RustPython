pub(crate) use _queue::module_def;

#[pymodule(name = "_queue")]
mod _queue {
    use alloc::collections::VecDeque;
    use core::time::Duration;
    use std::time::Instant;

    use parking_lot::Condvar;

    const POLL_INTERVAL: Duration = Duration::from_millis(100);

    use crate::vm::{
        PyObjectRef, PyResult, VirtualMachine, builtins::PyException, class::StaticType,
        function::OptionalArg, types::Constructor,
    };
    use rustpython_common::lock::PyMutex;
    use rustpython_vm::types::DefaultConstructor;

    #[pyattr]
    #[pyclass(name = "SimpleQueue")]
    #[derive(Debug, PyPayload, Default)]
    struct PySimpleQueue {
        queue: PyMutex<VecDeque<PyObjectRef>>,
        cvar: Condvar,
    }

    impl DefaultConstructor for PySimpleQueue {}

    #[derive(FromArgs)]
    struct PyFuncPutArgs {
        #[pyarg(any)]
        item: PyObjectRef,
        #[pyarg(any, optional)]
        _block: OptionalArg<PyObjectRef>,
        #[pyarg(any, optional)]
        _timeout: OptionalArg<PyObjectRef>,
    }

    #[derive(FromArgs)]
    struct PyFuncGetArgs {
        #[pyarg(any, optional)]
        block: OptionalArg<PyObjectRef>,
        #[pyarg(any, optional)]
        timeout: OptionalArg<PyObjectRef>,
    }

    #[pyclass(with(Constructor), flags(BASETYPE))]
    impl PySimpleQueue {
        #[pymethod]
        fn qsize(&self) -> usize {
            (*self.queue.lock()).len()
        }

        #[pymethod]
        fn empty(&self) -> bool {
            (*self.queue.lock()).is_empty()
        }

        #[pymethod]
        fn put_nowait(&self, x: PyObjectRef) {
            (*self.queue.lock()).push_back(x.clone());
            self.cvar.notify_one();
        }

        #[pymethod]
        fn get_nowait(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match (*self.queue.lock()).pop_front() {
                Some(value) => Ok(value),
                None => Err(vm.new_exception(Empty::static_type().to_owned(), vec![])),
            }
        }

        #[pymethod]
        fn put(&self, args: PyFuncPutArgs) {
            let PyFuncPutArgs {
                item,
                _block: _,
                _timeout: _,
            } = args;

            (*self.queue.lock()).push_back(item.clone());
            self.cvar.notify_one();
        }

        #[pymethod]
        fn get(&self, args: PyFuncGetArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let PyFuncGetArgs {
                block: block_obj,
                timeout: timeout_obj,
            } = args;

            let blocking: bool = match block_obj {
                OptionalArg::Present(value) => value.clone().try_to_bool(vm)?,
                OptionalArg::Missing => true,
            };

            if !blocking {
                match (*self.queue.lock()).pop_front() {
                    Some(value) => Ok(value),
                    None => Err(vm.new_exception(Empty::static_type().to_owned(), vec![])),
                }
            } else {
                let timeout = match timeout_obj {
                    OptionalArg::Present(value) => value.clone().try_float(vm)?.to_f64(),
                    OptionalArg::Missing => 0.0,
                };
                if timeout < 0.0 {
                    return Err(vm.new_value_error("timeout < 0"));
                }

                let mut q = self.queue.lock();

                if timeout > 0.0 {
                    let start = Instant::now();
                    let timeout = Duration::from_millis((timeout * 1000.0 + 0.5) as u64);

                    loop {
                        let result = self.cvar.wait_while_for(
                            &mut q,
                            |q: &mut VecDeque<PyObjectRef>| q.is_empty(),
                            POLL_INTERVAL,
                        );
                        vm.check_signals()?;
                        if !result.timed_out() {
                            break;
                        }

                        if start.elapsed() > timeout {
                            return Err(vm.new_exception(Empty::static_type().to_owned(), vec![]));
                        }
                    }

                    match (q).pop_front() {
                        Some(value) => Ok(value),
                        None =>
                        //should be impossible; panic?
                        {
                            Err(vm.new_exception(Empty::static_type().to_owned(), vec![]))
                        }
                    }
                } else {
                    loop {
                        let result = self.cvar.wait_while_for(
                            &mut q,
                            |q: &mut VecDeque<PyObjectRef>| q.is_empty(),
                            POLL_INTERVAL,
                        );
                        vm.check_signals()?;
                        if !result.timed_out() {
                            break;
                        }
                    }

                    match (q).pop_front() {
                        Some(value) => Ok(value),
                        None =>
                        //should be impossible; panic?
                        {
                            Err(vm.new_exception(Empty::static_type().to_owned(), vec![]))
                        }
                    }
                }
            }
        }
    }

    #[pyattr]
    #[pyexception(name = "Empty", base = PyException, impl)]
    #[derive(Debug)]
    #[repr(transparent)]
    struct Empty(PyException);
}
