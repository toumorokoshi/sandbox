import pickle
from dataclasses import dataclass


@dataclass
class ExampleDataClass:
    name: str
    age: int


# Create an instance of the dataclass
example = ExampleDataClass(name="John Doe", age=30)

# Pickle the dataclass instance
with open("example.pickle", "wb") as file:
    pickle.dump(example, file)

# Unpickle the dataclass instance
with open("example.pickle", "rb") as file:
    unpickled_example = pickle.load(file)

# Print the unpickled dataclass instance
print(unpickled_example)
