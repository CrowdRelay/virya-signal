import copy
import unittest

import google_play_rollout as rollout


class GooglePlayRolloutTests(unittest.TestCase):
    def track(self):
        return {
            "track": "production",
            "releases": [
                {"name": "old", "versionCodes": ["100"], "status": "completed"},
                {
                    "name": "Virya Signal 0.4.2",
                    "versionCodes": ["100000321"],
                    "status": "inProgress",
                    "userFraction": 0.10,
                    "releaseNotes": [{"language": "pl-PL", "text": "test"}],
                },
            ],
        }

    def test_advances_staged_release_and_preserves_metadata(self):
        updated, receipt = rollout._updated_track(self.track(), "25", "100000321")
        active = updated["releases"][1]
        self.assertEqual(active["status"], "inProgress")
        self.assertEqual(active["userFraction"], 0.25)
        self.assertEqual(active["releaseNotes"][0]["language"], "pl-PL")
        self.assertEqual(receipt["nextUserFraction"], 0.25)

    def test_completion_removes_user_fraction(self):
        updated, receipt = rollout._updated_track(self.track(), "100", None)
        active = updated["releases"][1]
        self.assertEqual(active["status"], "completed")
        self.assertNotIn("userFraction", active)
        self.assertEqual(receipt["nextUserFraction"], 1.0)

    def test_halt_only_targets_active_release(self):
        updated, receipt = rollout._updated_track(self.track(), "halt", "100000321")
        self.assertEqual(updated["releases"][0]["status"], "completed")
        self.assertEqual(updated["releases"][1]["status"], "halted")
        self.assertNotIn("userFraction", updated["releases"][1])
        self.assertEqual(receipt["previousUserFraction"], 0.10)

    def test_refuses_rollout_regression(self):
        track = self.track()
        track["releases"][1]["userFraction"] = 0.50
        with self.assertRaisesRegex(RuntimeError, "non-forward"):
            rollout._updated_track(track, "25", None)

    def test_expected_version_code_pins_target(self):
        with self.assertRaisesRegex(RuntimeError, "no active staged"):
            rollout._updated_track(self.track(), "50", "999")

    def test_refuses_ambiguous_active_releases(self):
        track = self.track()
        second = copy.deepcopy(track["releases"][1])
        second["versionCodes"] = ["100000322"]
        track["releases"].append(second)
        with self.assertRaisesRegex(RuntimeError, "multiple active"):
            rollout._updated_track(track, "50", None)


if __name__ == "__main__":
    unittest.main()
