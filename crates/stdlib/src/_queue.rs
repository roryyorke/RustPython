pub(crate) use _queue::module_def;

#[pymodule(name = "_queue")]
mod _queue {
    use crate::{
        vm::{
            PyObjectRef, PyResult, VirtualMachine,
            types::{Constructor},
        }
    };
    use rustpython_vm::types::DefaultConstructor;
    use rustpython_common::lock::PyMutex;

    #[pyattr]
    #[pyclass(name = "SimpleQueue")]
    #[derive(Debug, PyPayload, Default)]
    pub struct PySimpleQueue {
        queue: PyMutex<Vec<PyObjectRef>>,
        size: PyMutex<usize>,
    }

    impl DefaultConstructor for PySimpleQueue {}

    #[pyclass(with(Constructor), flags(BASETYPE))]
    impl PySimpleQueue {
        #[pymethod]
        pub fn qsize(&self) -> usize {
            *self.size.lock()
        }

        #[pymethod]
        pub fn empty(&self) -> bool {
            let size = self.size.lock();
            *size == 0
        }

        #[pymethod]
        pub fn put(&self, x: PyObjectRef) {
            *self.size.lock() += 1;
            (*self.queue.lock()).push(x.clone());
        }

        #[pymethod]
        pub fn get(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match (*self.queue.lock()).pop() {
                Some(value) => Ok(value),
                None => Err(vm.new_value_error("queue is empty".to_owned()))
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
}
