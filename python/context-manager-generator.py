import unittest
import sys
from unittest.mock import patch


class Config(dict):
    pass


def get_global_config(self, key):
    return self[key]


# just monkey-patching a function to mock over.
sys.get_global_config = get_global_config

GLOBAL_CONFIG = Config({"foo": 1})


class MyTestCase(unittest.TestCase):

    _mock_global_config = {"foo": 2}

    def setUp(self):
        self._patcher = patch("sys.get_global_config", new=self._mock_global_config.get)
        self._patcher.start()

    def tearDown(self):
        self._patcher.stop()

    def test_foo(self):
        # sys.__module__ is just a roundabout way of emulating importing and
        # referencing variables from a different module.
        self.assertEqual(sys.get_global_config("foo"), 4)


class MyTestCaseY(MyTestCase):
    """Identical to X, but with a new config.

    This allows test code to be shared across both,
    but modified configuration to a different global.
    """

    _mock_global_config = {"foo": 3}


if __name__ == "__main__":
    unittest.main()
