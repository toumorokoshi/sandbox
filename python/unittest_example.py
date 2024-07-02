import unittest

# Define your test class
class MyTestCase(unittest.TestCase):
    def test_addition(self):
        self.assertEqual(2 + 2, 4)

    def test_subtraction(self):
        self.assertEqual(5 - 3, 2)

    def test_multiplication(self):
        self.assertEqual(3 * 5, 12)

# Define the main function to execute the tests
def main():
    unittest.main()

# Execute the tests when the script is run
if __name__ == '__main__':
    main()