import json
import tempfile
import unittest
from pathlib import Path
from import_board_assets import compile_assets, validate


def fixture():
    return dict(version=1, coordinates="ricochet-world", board="ferrite", objects=[
        dict(name="back_toy", role="scenery", ink=10,
             vertices=[[40, 5, 1], [50, 5, 1], [45, 10, 6]], triangles=[[0, 1, 2]])])


class PipelineTests(unittest.TestCase):
    def test_compile_and_hash(self):
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder)
            (source / "ferrite.json").write_text(json.dumps(fixture()), encoding="utf-8")
            code, receipt = compile_assets(source)
            self.assertIn('"ferrite" =>', code)
            self.assertIn("ink: 10, floor: false", code)
            self.assertEqual(receipt[0]["triangles"], 1)
            self.assertEqual(len(receipt[0]["sha256"]), 64)
            self.assertEqual((code, receipt), compile_assets(source))

    def test_bad_geometry_and_metadata(self):
        for field, value in [("ink", 13), ("role", "collision"),
                             ("triangles", [[0, 1, 99]]),
                             ("triangles", [[0, 0, 1]]),
                             ("vertices", [[float("nan"), 5, 1]]),
                             ("vertices", [[40, 50, 1], [50, 50, 1], [45, 60, 6]])]:
            data = fixture()
            data["objects"][0][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                validate(data)

    def test_triangle_crossing_playfield_is_rejected(self):
        data = fixture()
        data["objects"][0]["vertices"] = [[10, 50, 1], [320, 50, 1], [45, 5, 6]]
        with self.assertRaisesRegex(ValueError, "playable area"):
            validate(data)

    def test_flush_artwork_and_budget(self):
        data = fixture()
        obj = data["objects"][0]
        obj.update(role="artwork", vertices=[[40, 50, .2], [50, 50, .2], [45, 60, .2]])
        self.assertEqual(validate(data), 1)
        obj["vertices"][0][2] = 1
        with self.assertRaisesRegex(ValueError, "flush"):
            validate(data)
        data = fixture()
        data["objects"][0]["triangles"] *= 4001
        with self.assertRaisesRegex(ValueError, "budget"):
            validate(data)

    def test_duplicate_board_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder)
            for name in ("a", "b"):
                (source / f"{name}.json").write_text(json.dumps(fixture()), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Duplicate board"):
                compile_assets(source)


if __name__ == "__main__":
    unittest.main()
