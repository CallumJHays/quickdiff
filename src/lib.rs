use pyo3::{prelude::*, types::*, PyTypeInfo};

type KeyPath = Vec<Py<PyAny>>;

#[pyclass]
#[derive(Clone)]
struct ValChange(
    #[pyo3(get, name="path")]
    KeyPath,
    #[pyo3(get, name="a")]
    Py<PyAny>,
    #[pyo3(get, name="b")]
    Py<PyAny>);

#[pyclass]
#[derive(Clone)]
struct TypeAndValChange(
    #[pyo3(get, name="path")]
    KeyPath,
    #[pyo3(get, name="a")]
    Py<PyAny>,
    #[pyo3(get, name="b")]
    Py<PyAny>);

#[pyclass]
#[derive(Clone)]
struct DictDiff(
    #[pyo3(get, name="path")]
    KeyPath,
    #[pyo3(get, name="key")]
    Py<PyAny>,
    #[pyo3(get, name="val")]
    Py<PyAny>);

#[pyclass]
#[derive(Clone)]
struct IterLenMismatch(
    #[pyo3(get, name="path")]
    KeyPath,
    #[pyo3(get, name="a_len")]
    usize,
    #[pyo3(get, name="b_len")]
    usize);


#[derive(Clone)]
enum Diff {
    ValChange(ValChange),
    TypeAndValChange(TypeAndValChange),
    DictItemAdded(DictDiff),
    DictItemRemoved(DictDiff),
    IterLenMismatch(IterLenMismatch)
}

#[derive(Default)]
#[pyclass] // pyclass(get_all) not available for some reason...
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
    iter_len_mismatch: Vec<IterLenMismatch>
}

#[pyfunction(name="quickdiff")]
fn python_entrypoint(a: &PyAny, b: &PyAny) -> Output {
    let mut output = Output::default();

    for d in quickdiff_dispatch(a, b, vec![]) {
        match d {
            Diff::ValChange(c) => output.val_changes.push(c),
            Diff::TypeAndValChange(c) => output.type_and_val_changes.push(c),
            Diff::DictItemAdded(c) => output.dict_items_added.push(c),
            Diff::DictItemRemoved(c) => output.dict_items_removed.push(c),
            Diff::IterLenMismatch(c) => output.iter_len_mismatch.push(c)
        };
    }
    
    output
}


fn quickdiff_dispatch<'a>(a: &'a PyAny, b: &'a PyAny, keypath: KeyPath) -> Vec<Diff> {
    if let Ok((a, b)) = downcast_both::<PyString>(a, b) {
        let (av, bv) = extract_both::<String>(a, b).unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)

    } else if let Ok((a, b)) = downcast_both::<PyInt>(a, b) {
        let (av, bv) = extract_both::<i128>(a, b).unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)

    } else if let Ok((a, b)) = downcast_both::<PyFloat>(a, b) {
        let (av, bv) = extract_both::<f64>(a, b).unwrap();
        quickdiff_primitive(a, b, av, bv, keypath)

    } else if let Ok((av, bv)) = cast_both_as::<PyMapping>(a, b) {
        quickdiff_map(av, bv, keypath)
        
    } else if let Ok((av, bv)) = cast_both_as::<PySequence>(a, b) {
        quickdiff_iter(av.iter().unwrap(), bv.iter().unwrap(), keypath)

    } else if let Ok((av, bv)) = cast_both_as::<PyIterator>(a, b) {
        quickdiff_iter(av, bv, keypath)

    } else if a.is_none() && b.is_none() {
        vec![]
        
    } else {
        vec![Diff::TypeAndValChange(
            TypeAndValChange(keypath, a.into(), b.into())
        )]
    }
}


fn quickdiff_map<'a>(a: &'a PyMapping, b: &'a PyMapping, keypath: KeyPath) -> Vec<Diff>  {

    let items_added = a.items().unwrap().iter().unwrap().flat_map(
        |item_pair| {
            let tup = item_pair.unwrap().cast_as::<PyTuple>().unwrap();
            if let [ak, av] = *tup.as_slice() {
                if let Ok(bv) = b.get_item(ak) {
                    quickdiff_dispatch(av, bv, extend_keypath(&keypath, ak))
                } else {
                    let diff = DictDiff(keypath.clone(), ak.into(), av.into());
                    vec![Diff::DictItemRemoved(diff)]
                }
            } else {
                panic!("items() tuple should always be an iterator of 2-tuples")
            }
        }
    );

    let items_removed = b.items().unwrap().iter().unwrap().flat_map(
        |item_pair| {
            let tup = item_pair.unwrap().cast_as::<PyTuple>().unwrap();
            if let [bk, bv] = *tup.as_slice() {
                if !a.contains(bk).unwrap() {
                    let diff = DictDiff(keypath.clone(), bk.into(), bv.into());
                    vec![Diff::DictItemAdded(diff)]
                } else {
                    vec![]
                }
            } else {
                panic!("items() tuple should always be an iterator of 2-tuples")
            }
        }
    );

    items_added.chain(items_removed).collect()
    
}


fn quickdiff_iter<'a>(a: &'a PyIterator, b: &'a PyIterator, keypath: KeyPath) -> Vec<Diff> {

    let av: Vec<&PyAny> = a.map(Result::unwrap).collect();
    let bv: Vec<&PyAny> = b.map(Result::unwrap).collect();
    
    let el_results : Vec<Diff> = av.iter().zip(bv.iter()).enumerate().flat_map(
        |(idx, (a, b))| {
            let py = a.py();
            let path = extend_keypath(
                &keypath,
                idx.to_object(py).into_ref(py)
            );
            quickdiff_dispatch(a, b, path)
        }
    ).collect();
    
    if av.len() != bv.len() {
        let diff = Diff::IterLenMismatch(
            IterLenMismatch(keypath, av.len(), bv.len())
        );
        [el_results, vec![diff]].concat()
    } else {
        el_results
    }
}


fn quickdiff_primitive<'a, T: PartialEq>(a: &'a PyAny, b: &'a PyAny, av: T, bv: T, keypath: KeyPath) -> Vec<Diff> {
    if av != bv {
        vec![Diff::ValChange(ValChange(keypath, a.into(), b.into()))]
    } else {
        vec![]
    }
}

fn downcast_both<'a, T: PyTryFrom<'a> + PyTypeInfo>(a: &'a PyAny, b: &'a PyAny) -> PyResult<(&'a T, &'a T)> {
    Ok((a.downcast::<T>()?, b.downcast::<T>()?))
}

fn cast_both_as<'a, T: PyTryFrom<'a>>(a: &'a PyAny, b: &'a PyAny) -> PyResult<(&'a T, &'a T)> {
    Ok((a.cast_as::<T>()?, b.cast_as::<T>()?))
}


fn extract_both<'a, T: FromPyObject<'a>>(a: &'a PyAny, b: &'a PyAny) -> PyResult<(T, T)> {
    Ok((a.extract()?, b.extract()?))
}


/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "quickdiff")]
fn quickdiff_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(python_entrypoint, m)?)?;
    m.add_class::<Output>()?;
    Ok(())
}

fn extend_keypath(keypath: &KeyPath, val: &PyAny) -> KeyPath {
    [keypath.clone(), vec![val.into()]].concat()
}

