use frigg_core::{Action, CheckResult, Frigg};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::path::PathBuf;

#[pyclass]
struct PyFrigg {
    inner: Frigg,
}

#[pymethods]
impl PyFrigg {
    #[staticmethod]
    fn from_config(rules_path: &str, log_path: &str) -> PyResult<Self> {
        let inner = Frigg::from_config(&PathBuf::from(rules_path), &PathBuf::from(log_path))
            .map_err(|e| PyRuntimeError::new_err(e))?;
        Ok(Self { inner })
    }

    fn check<'py>(&mut self, action: &Bound<'py, PyDict>, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let name: String = action
            .get_item("name")?
            .ok_or_else(|| PyRuntimeError::new_err("action must have 'name' key"))?
            .extract()?;

        let params: HashMap<String, serde_json::Value> = if let Some(p) = action.get_item("params")? {
            let dict: HashMap<String, String> = p.extract().unwrap_or_default();
            dict.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect()
        } else {
            HashMap::new()
        };

        let a = Action { name, params };
        let result = self.inner.check_with(&a, |_| false);

        let out = PyDict::new(py);
        match result {
            CheckResult::Allowed => { out.set_item("decision", "allowed")?; }
            CheckResult::Blocked { rule_id, reason } => {
                out.set_item("decision", "blocked")?;
                out.set_item("rule_id", rule_id)?;
                out.set_item("reason", reason)?;
            }
            CheckResult::Warned { rule_id, reason } => {
                out.set_item("decision", "warned")?;
                out.set_item("rule_id", rule_id)?;
                out.set_item("reason", reason)?;
            }
        }
        Ok(out)
    }
}

#[pymodule]
fn _frigg(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFrigg>()?;
    Ok(())
}
