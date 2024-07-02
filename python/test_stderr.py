import subprocess


result = subprocess.run(["bash", "-c", ">&2 echo error"], stderr=None)
