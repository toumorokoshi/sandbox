import subprocess
import time

subprocess.run(["bash", "-c", "for i in {1..10}; do echo 'hello'; sleep 1; done"])
print("Script executed successfully!")
