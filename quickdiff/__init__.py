
from dataclasses import dataclass
from typing import Any, NamedTuple
from .quickdiff import quickdiff as native_quickdiff # type: ignore

class ValChange(NamedTuple):
    path: list[Any]
    a: Any
    b: Any

class TypeAndValChange(NamedTuple):
    path: list[Any]
    a: Any
    b: Any

class DictDiff(NamedTuple):
    path: list[Any]
    key: Any
    val: Any

class IterLenMismatch(NamedTuple):
    path: list[Any]
    a_len: Any
    b_len: Any

@dataclass
class DiffReport:
    val_changes: list[ValChange]
    type_and_val_changes: list[TypeAndValChange]
    dict_items_added: list[DictDiff]
    dict_items_removed: list[DictDiff]
    iter_len_mismatch: list[IterLenMismatch]

def quickdiff(a: Any, b: Any) -> DiffReport:
    """
    
    Arguments:
        a {Any} -- 
    """
    out = native_quickdiff(a, b) # type: ignore
    return DiffReport(
        val_changes=[ValChange(x.path, x.a, x.b) for x in out.val_changes], # type: ignore
        type_and_val_changes=[TypeAndValChange(x.path, x.a, x.b) for x in out.type_and_val_changes], # type: ignore
        dict_items_added=[DictDiff(x.path, x.key, x.val) for x in out.dict_items_added], # type: ignore
        dict_items_removed=[DictDiff(x.path, x.key, x.val) for x in out.dict_items_removed], # type: ignore
        iter_len_mismatch=[IterLenMismatch(x.path, x.a_len, x.b_len) for x in out.iter_len_mismatch], # type: ignore
    )

