import os
import pty
import subprocess
import threading
import time
import unittest


class SerialProbeTest(unittest.TestCase):
    def test_probe_reads_scale_frame_from_pseudo_terminal(self) -> None:
        master_fd, slave_fd = pty.openpty()
        slave_name = os.ttyname(slave_fd)
        os.close(slave_fd)

        def writer() -> None:
            deadline = time.time() + 3.0
            sent = 0
            while time.time() < deadline and sent < 5:
                try:
                    os.write(master_fd, b"1.250 kg ST\r")
                    sent += 1
                except OSError:
                    time.sleep(0.05)
                    continue
                time.sleep(0.05)

        thread = threading.Thread(target=writer, daemon=True)
        thread.start()

        try:
            result = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "--bin",
                    "rp-scale-probe-serial",
                    "--",
                    slave_name,
                    "9600",
                    "1200",
                    "kg",
                ],
                cwd=os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
                text=True,
                capture_output=True,
                check=True,
                timeout=10,
            )
        finally:
            os.close(master_fd)

        self.assertIn("parsed_weight=true", result.stdout)
        self.assertIn("has_data=true", result.stdout)


if __name__ == "__main__":
    unittest.main()
