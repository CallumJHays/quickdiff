use pyo3::prelude::*;
use pyo3::types::*;

type KeyPath = Vec<Py<PyAny>>;

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct ValChange {
    #[pyo3(get)]
    path: KeyPath,
    #[pyo3(get)]
    a: Py<PyAny>,
    #[pyo3(get)]
    b: Py<PyAny>,
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct TypeAndValChange {
    #[pyo3(get)]
    path: KeyPath,
    #[pyo3(get)]
    a: Py<PyAny>,
    #[pyo3(get)]
    b: Py<PyAny>,
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct DictDiff {
    #[pyo3(get)]
    path: KeyPath,
    #[pyo3(get)]
    key: Py<PyAny>,
    #[pyo3(get)]
    val: Py<PyAny>,
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct IterLenMismatch {
    #[pyo3(get)]
    path: KeyPath,
    #[pyo3(get)]
    a_len: usize,
    #[pyo3(get)]
    b_len: usize,
}

#[derive(Clone)]
enum Diff {
    ValChange(ValChange),
    TypeAndValChange(TypeAndValChange),
    DictItemAdded(DictDiff),
    DictItemRemoved(DictDiff),
    IterLenMismatch(IterLenMismatch),
}

#[derive(Default)]
#[pyclass]
struct Output {
    #[pyo3(get)]
    val_changes: Vec<ValChange>,
    #[pyo3(get)]
    type_and_val_changes: Vec<TypeAndValChange>,
    #[pyo3(get)]
    dict_items_added: Vec<DictDiff>,
    #[pyo3(get)]
    dict_items_removed: Vec<DictDiff>,
    #[pyo3(get)]
    iter_len_mismatch: Vec<IterLenMismatch>,
}

#[pyfunction(name = "quickdiff")]
fn python_entrypoint(a: Bound<'_, PyAny>, b: Bound<'_, PyAny>) -> Output {
    let mut output = Output::default();
    for d in quickdiff_dispatch(&a, &b, vec![]) {
        match d {
            Diff::ValChange(c) => output.val_changes.push(c),
            Diff::TypeAndValChange(c) => output.type_and_val_changes.push(c),
            Diff::DictItemAdded(c) => output.dict_items_added.push(c),
            Diff::DictItemRemoved(c) => output.dict_items_removed.push(c),
            Diff::IterLenMismatch(c) => output.iter_len_mismatch.push(c),
        }
    }
    output
}

fn quickdiff_dispatch<'py>(
    a: &Bound<'py, PyAny>,
    b: &Bound<'py, PyAny>,
    keypath: KeyPath,
) -> Vec<Diff> {
    if let (Ok(as_), Ok(bs)) = (a.cast::<PyString>(), b.cast::<PyString>()) {
        let av = as_.extract::<String>().unwrap();
        let bv = bs.extract::<String>().unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)
    } else if let (Ok(ai), Ok(bi)) = (a.cast::<PyInt>(), b.cast::<PyInt>()) {
        let av = ai.extract::<i128>().unwrap();
        let bv = bi.extract::<i128>().unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)
    } else if let (Ok(af), Ok(bf)) = (a.cast::<PyFloat>(), b.cast::<PyFloat>()) {
        let av = af.extract::<f64>().unwrap();
        let bv = bf.extract::<f64>().unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)
    } else if let (Ok(am), Ok(bm)) = (a.cast::<PyMapping>(), b.cast::<PyMapping>()) {
        quickdiff_map(&am, &bm, keypath)
    } else if let (Ok(aseq), Ok(bseq)) = (a.cast::<PySequence>(), b.cast::<PySequence>()) {
        let ai = aseq.as_any().try_iter().unwrap();
        let bi = bseq.as_any().try_iter().unwrap();
        quickdiff_iter(&ai, &bi, keypath)
    } else if let (Ok(ai), Ok(bi)) = (a.cast::<PyIterator>(), b.cast::<PyIterator>()) {
        quickdiff_iter(&ai, &bi, keypath)
    } else if a.is_none() && b.is_none() {
        vec![]
    } else {
        vec![Diff::TypeAndValChange(TypeAndValChange {
            path: keypath,
            a: a.clone().unbind(),
            b: b.clone().unbind(),
        })]
    }
}

fn quickdiff_map<'py>(
    a: &Bound<'py, PyMapping>,
    b: &Bound<'py, PyMapping>,
    keypath: KeyPath,
) -> Vec<Diff> {
    let a_items = a.items().unwrap();
    let items_added: Vec<Diff> = a_items
        .iter()
        .flat_map(|item| {
            let tup = item.cast_into::<PyTuple>().unwrap();
            let ak = tup.get_item(0).unwrap();
            let av = tup.get_item(1).unwrap();
            if let Ok(bv) = b.get_item(ak.clone()) {
                quickdiff_dispatch(&av, &bv, extend_keypath(&keypath, &ak))
            } else {
                vec![Diff::DictItemRemoved(DictDiff {
                    path: keypath.clone(),
                    key: ak.unbind(),
                    val: av.unbind(),
                })]
            }
        })
        .collect();

    let b_items = b.items().unwrap();
    let items_removed: Vec<Diff> = b_items
        .iter()
        .flat_map(|item| {
            let tup = item.cast_into::<PyTuple>().unwrap();
            let bk = tup.get_item(0).unwrap();
            let bv = tup.get_item(1).unwrap();
            if !a.contains(bk.clone()).unwrap() {
                vec![Diff::DictItemAdded(DictDiff {
                    path: keypath.clone(),
                    key: bk.unbind(),
                    val: bv.unbind(),
                })]
            } else {
                vec![]
            }
        })
        .collect();

    [items_added, items_removed].concat()
}

fn quickdiff_iter(
    a: &Bound<'_, PyIterator>,
    b: &Bound<'_, PyIterator>,
    keypath: KeyPath,
) -> Vec<Diff> {
    let av= a.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
    let bv= b.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
    
    let el_results: Vec<Diff> = av
        .iter()
        .zip(bv.iter())
        .enumerate()
        .flat_map(|(idx, (a, b))| {
            let py = a.py();
            let idx_obj = idx.into_pyobject(py).unwrap();
            let path = extend_keypath(&keypath, idx_obj.as_any());
            quickdiff_dispatch(a, b, path)
        })
        .collect();

    if av.len() != bv.len() {
        [
            el_results,
            vec![Diff::IterLenMismatch(IterLenMismatch {
                path: keypath,
                a_len: av.len(),
                b_len: bv.len(),
            })],
        ]
        .concat()
    } else {
        el_results
    }
}

fn quickdiff_primitive<T: PartialEq>(
    a: &Bound<'_, PyAny>,
    b: &Bound<'_, PyAny>,
    av: T,
    bv: T,
    keypath: KeyPath,
) -> Vec<Diff> {
    if av != bv {
        vec![Diff::ValChange(ValChange {
            path: keypath,
            a: a.clone().unbind(),
            b: b.clone().unbind(),
        })]
    } else {
        vec![]
    }
}

fn extend_keypath(keypath: &KeyPath, val: &Bound<'_, PyAny>) -> KeyPath {
    let mut new = keypath.clone();
    new.push(val.clone().unbind());
    new
}

#[pymodule]
#[pyo3(name = "quickdiff")]
fn quickdiff_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(python_entrypoint, m)?)?;
    m.add_class::<Output>()?;
    Ok(())
}
