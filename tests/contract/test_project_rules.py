from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MAX_LINES = 500


class ProjectRulesTest(unittest.TestCase):
    def test_source_files_stay_under_500_lines(self) -> None:
        checked_suffixes = {".rs", ".md", ".py", ".toml"}
        ignored_dirs = {".git", "target"}
        offenders: list[str] = []

        for path in ROOT.rglob("*"):
            if any(part in ignored_dirs for part in path.parts):
                continue
            if not path.is_file() or path.suffix not in checked_suffixes:
                continue
            line_count = len(path.read_text(encoding="utf-8").splitlines())
            if line_count > MAX_LINES:
                offenders.append(f"{path.relative_to(ROOT)}: {line_count}")

        self.assertEqual([], offenders)


if __name__ == "__main__":
    unittest.main()
