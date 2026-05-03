pub(crate) use _queue::module_def;

#[pymodule(name = "_queue")]
mod _queue {
    use std::collections::VecDeque;

    use crate::{
        vm::{
            PyObjectRef, PyResult, VirtualMachine,
            builtins::PyException,
            types::Constructor,
            class::StaticType,
        }
    };
    use rustpython_vm::types::DefaultConstructor;
    use rustpython_common::lock::PyMutex;

    #[pyattr]
    #[pyclass(name = "SimpleQueue")]
    #[derive(Debug, PyPayload, Default)]
    pub struct PySimpleQueue {
        queue: PyMutex<VecDeque<PyObjectRef>>,
    }

    impl DefaultConstructor for PySimpleQueue {}

    #[pyclass(with(Constructor), flags(BASETYPE))]
    impl PySimpleQueue {
        #[pymethod]
        pub fn qsize(&self) -> usize {
            (*self.queue.lock()).len()
        }

        #[pymethod]
        pub fn empty(&self) -> bool {
            (*self.queue.lock()).len() == 0
        }

        #[pymethod]
        pub fn put_nowait(&self, x: PyObjectRef) {
            (*self.queue.lock()).push_back(x.clone());
        }

        #[pymethod]
        pub fn get_nowait(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match (*self.queue.lock()).pop_front() {
                Some(value) => Ok(value),
                None => Err(vm.new_exception(
                    Empty::static_type().to_owned(), vec![]))
            }
        }

        #[pymethod]
        pub fn throw_if_even(&self, x: i32, vm: &VirtualMachine) -> PyResult<i32> {
            if x%2 == 0 {
                Err(vm.new_value_error("x is even".to_owned()))
            } else {
                Ok(99)
            }
        }

        // #[pymethod]
        // pub fn put(&mut self, x: PyObjectRef, _block: bool, _timeout: f64) {
        //     self.queue.push(x);
        // }

        // #[pymethod]
        // pub fn get(&mut self) -> Option<PyObjectRef> {
        //     self.queue.pop()
        // }
    }

    #[pyattr]
    #[pyexception(name = "Empty", base = PyException, impl)]
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct Empty(PyException);

}
