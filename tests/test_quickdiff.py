from typing import NamedTuple
import unittest
from quickdiff import *
from rich import print
import sys


class TestQuickDiff(unittest.TestCase):

    def assert_report(
        self,
        report: DiffReport,
        val_changes: list[ValChange] = [],
        type_and_val_changes: list[TypeAndValChange] = [],
        dict_items_added: list[DictDiff] = [],
        dict_items_removed: list[DictDiff] = [],
        iter_len_mismatch: list[IterLenMismatch] = []
    ):
        self.assertEqual(val_changes, report.val_changes)
        self.assertEqual(type_and_val_changes, report.type_and_val_changes)
        self.assertEqual(dict_items_added, report.dict_items_added)
        self.assertEqual(dict_items_removed, report.dict_items_removed)
        self.assertEqual(iter_len_mismatch, report.iter_len_mismatch)


    def test_int_eq_int(self):
        self.assert_report(quickdiff(2, 2)) # no diff
        self.assert_report(quickdiff(1, 1)) # no diff
        self.assert_report(quickdiff(-987, -987)) # no diff


    def test_float_eq_float(self):
        self.assert_report(quickdiff(2.0, 2.0)) # no diff
        self.assert_report(quickdiff(1.0, 1.0)) # no diff
        self.assert_report(quickdiff(-987.0, -987.0)) # no diff


    def test_str_eq_str(self):
        self.assert_report(quickdiff("hello", "hello"))
        self.assert_report(quickdiff("s-=26dasd", "s-=26dasd"))
        self.assert_report(quickdiff("l&kisn123jfas", "l&kisn123jfas"))

    def test_bool_eq_bool(self):
        self.assert_report(quickdiff(True, True))
        self.assert_report(quickdiff(False, False))
    
    def test_none_eq_none(self):
        self.assert_report(quickdiff(None, None))
    
    def test_int_ne_int(self):
        self.assert_report(quickdiff(2, 3), val_changes=[ValChange([], 2, 3)])
        self.assert_report(quickdiff(1, 0), val_changes=[ValChange([], 1, 0)])
        self.assert_report(quickdiff(-987, -986), val_changes=[ValChange([], -987, -986)])
    
    def test_float_ne_float(self):
        self.assert_report(quickdiff(2.0, 3.0), val_changes=[ValChange([], 2.0, 3.0)])
        self.assert_report(quickdiff(1.0, 0.0), val_changes=[ValChange([], 1.0, 0.0)])
        self.assert_report(quickdiff(-987.0, -986.0), val_changes=[ValChange([], -987.0, -986.0)])
    
    def test_str_ne_str(self):
        self.assert_report(quickdiff("hello", "helloo"), val_changes=[ValChange([], "hello", "helloo")])
        self.assert_report(quickdiff("s-=26dasd", "s-=26dasd1"), val_changes=[ValChange([], "s-=26dasd", "s-=26dasd1")])
        self.assert_report(quickdiff("l&kisn123jfas", "1+l&kisn123jfas"), val_changes=[ValChange([], "l&kisn123jfas", "1+l&kisn123jfas")])

    def test_tuple_eq_tuple(self):
        self.assert_report(quickdiff((1, 2, 3), (1, 2, 3)))
    
    def test_tuple_neq_tuple(self):
        self.assert_report(quickdiff((1, 2, 3), (1, 2, 4)), val_changes=[ValChange([2], 3, 4)])
        self.assert_report(quickdiff((2, 2, 3), (1, 2, 3)), val_changes=[ValChange([0], 2, 1)])

    def test_iter_len_mismatch(self):
        self.assert_report(quickdiff((1, 2, 3), (1, 2)), iter_len_mismatch=[IterLenMismatch([], 3, 2)])
        self.assert_report(quickdiff((1, 2), (1, 2, 3)), iter_len_mismatch=[IterLenMismatch([], 2, 3)])
    
    def test_list_eq_list(self):
        self.assert_report(quickdiff([1, 2, 3], [1, 2, 3]))
    
    def test_list_neq_list(self):
        self.assert_report(quickdiff([1, 2, 3], [1, 2, 4]), val_changes=[ValChange([2], 3, 4)])
        self.assert_report(quickdiff([2, 2, 3], [1, 2, 3]), val_changes=[ValChange([0], 2, 1)])
    
    def test_dict_eq_dict(self):
        self.assert_report(quickdiff({"a": 1, "b": 2}, {"a": 1, "b": 2}))
    
    def test_dict_neq_dict(self):
        self.assert_report(quickdiff({"a": 1, "b": 2}, {"a": 1, "b": 3}), val_changes=[ValChange(["b"], 2, 3)])
        self.assert_report(
            quickdiff({"a": 1, "b": 2}, {"a": 1, "c": 3}),
            dict_items_removed=[DictDiff([], "b", 2)],
            dict_items_added=[DictDiff([], "c", 3)]
        )
        self.assert_report(quickdiff({"a": 1}, {"a": 1, "b": 2}), dict_items_added=[DictDiff([], "b", 2)])
        self.assert_report(quickdiff({"a": 1, "b": 2}, {"a": 1}), dict_items_removed=[DictDiff([], "b", 2)])

    def test_type_and_val_change(self):
        self.assert_report(quickdiff(1, 1.0), type_and_val_changes=[TypeAndValChange([], 1, 1.0)])
        self.assert_report(quickdiff(1.0, 1), type_and_val_changes=[TypeAndValChange([], 1.0, 1)])
        self.assert_report(quickdiff(1, "1"), type_and_val_changes=[TypeAndValChange([], 1, "1")])
        self.assert_report(quickdiff("1", 1), type_and_val_changes=[TypeAndValChange([], "1", 1)])
        self.assert_report(quickdiff(1.0, "1"), type_and_val_changes=[TypeAndValChange([], 1.0, "1")])
        self.assert_report(quickdiff("1", 1.0), type_and_val_changes=[TypeAndValChange([], "1", 1.0)])
    
    def test_nested_dict_eq(self):
        self.assert_report(quickdiff({"a": {"b": 1}}, {"a": {"b": 1}}))

    def test_nested_dict_neq(self):
        self.assert_report(quickdiff({"a": {"b": 1}}, {"a": {"b": 2}}), val_changes=[ValChange(["a", "b"], 1, 2)])
        self.assert_report(quickdiff({"a": {"b": 1}}, {"a": {"c": 2}}), dict_items_removed=[DictDiff(["a"], "b", 1)], dict_items_added=[DictDiff(["a"], "c", 2)])
        self.assert_report(quickdiff({"a": {"b": 1}}, {"a": {"b": 1, "c": 2}}), dict_items_added=[DictDiff(["a"], "c", 2)])
        self.assert_report(quickdiff({"a": {"b": 1, "c": 2}}, {"a": {"b": 1}}), dict_items_removed=[DictDiff(["a"], "c", 2)])
        self.assert_report(quickdiff({"a": {"b": 1, "c": 2}}, {"a": {"b": 1, "c": 3}}), val_changes=[ValChange(["a", "c"], 2, 3)])
        self.assert_report(quickdiff({"a": {"b": 1, "c": 2}}, {"a": {"b": 1, "c": 2, "d": 3}}), dict_items_added=[DictDiff(["a"], "d", 3)])
    
    def test_nested_list_eq(self):
        self.assert_report(quickdiff([1, [2, 3]], [1, [2, 3]]))
    
    def test_nested_list_neq(self):
        self.assert_report(quickdiff([1, [2, 3]], [1, [2, 4]]), val_changes=[ValChange([1, 1], 3, 4)])
        self.assert_report(quickdiff([1, [2, 3]], [1, [2, 3, 4]]), iter_len_mismatch=[IterLenMismatch([1], 2, 3)])
        self.assert_report(quickdiff([1, [2, 3, 4]], [1, [2, 3]]), iter_len_mismatch=[IterLenMismatch([1], 3, 2)])
        self.assert_report(quickdiff([1, [2, 3]], [1, [4, 3]]), val_changes=[ValChange([1, 0], 2, 4)])
    
    def test_maxsize_int_eq_maxsize_int(self):
        self.assert_report(quickdiff(444444444444444444444444444, 444444444444444444444444444))


class Report(NamedTuple):
    val_changes: list[ValChange]
    type_and_val_changes: list[TypeAndValChange]
    dict_items_added: list[DictDiff]
    dict_items_removed: list[DictDiff]
    iter_len_mismatch: list[IterLenMismatch]


def printreport(report: DiffReport):
    print(Report(
        report.val_changes,
        report.type_and_val_changes,
        report.dict_items_added,
        report.dict_items_removed,
        report.iter_len_mismatch
    ))

if __name__ == "__main__":
    unittest.main()

DiffReport(
    val_changes=[ValChange(path=[2], a=2, b=4)],
    type_and_val_changes=[],
    dict_items_added=[DictDiff(path=[], key=5, val=5), DictDiff(path=[], key=6, val=6)],
    dict_items_removed=[DictDiff(path=[], key=4, val=4)],
    iter_len_mismatch=[IterLenMismatch(path=[3], a_len=1, b_len=2)]
)