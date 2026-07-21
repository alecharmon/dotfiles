#!/usr/bin/env python3
import importlib.machinery
import importlib.util
import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_PATH = Path(__file__).with_name("sketchybar-prs")

loader = importlib.machinery.SourceFileLoader("sketchybar_prs", str(SCRIPT_PATH))
spec = importlib.util.spec_from_loader(loader.name, loader)
sketchybar_prs = importlib.util.module_from_spec(spec)
loader.exec_module(sketchybar_prs)


class SketchybarPrsTests(unittest.TestCase):
    def test_format_age_uses_largest_useful_unit(self):
        now = datetime(2026, 6, 23, 12, 0, tzinfo=timezone.utc)
        self.assertEqual(sketchybar_prs.format_age("2026-06-23T11:30:00Z", now=now), "30m")
        self.assertEqual(sketchybar_prs.format_age("2026-06-23T09:00:00Z", now=now), "3h")
        self.assertEqual(sketchybar_prs.format_age("2026-06-20T12:00:00Z", now=now), "3d")

    def test_normalize_pr_extracts_repo_status_and_links(self):
        raw = {
            "number": 42,
            "title": "Add widget",
            "url": "https://github.com/acme/widgets/pull/42",
            "createdAt": "2026-06-20T12:00:00Z",
            "updatedAt": "2026-06-22T12:00:00Z",
            "isDraft": True,
            "repository": {"nameWithOwner": "acme/widgets"},
        }
        pr = sketchybar_prs.normalize_pr(raw, "mine", now=datetime(2026, 6, 23, 12, 0, tzinfo=timezone.utc))
        self.assertEqual(pr["id"], "acme/widgets#42")
        self.assertEqual(pr["repo"], "acme/widgets")
        self.assertEqual(pr["displayName"], "acme/widgets")
        self.assertEqual(pr["age"], "3d")
        self.assertEqual(pr["status"], "draft")
        self.assertEqual(pr["openUrl"], "https://github.com/acme/widgets/pull/42")
        self.assertFalse(pr["isStack"])

    def test_build_cache_deduplicates_mine_and_review_requests(self):
        mine = [
            {"number": 1, "title": "Mine", "url": "https://github.com/acme/a/pull/1", "createdAt": "2026-06-22T12:00:00Z", "updatedAt": "2026-06-22T12:00:00Z", "isDraft": False, "repository": {"nameWithOwner": "acme/a"}},
        ]
        reviews = [
            {"number": 1, "title": "Mine", "url": "https://github.com/acme/a/pull/1", "createdAt": "2026-06-22T12:00:00Z", "updatedAt": "2026-06-22T12:00:00Z", "isDraft": False, "repository": {"nameWithOwner": "acme/a"}},
            {"number": 2, "title": "Review", "url": "https://github.com/acme/b/pull/2", "createdAt": "2026-06-21T12:00:00Z", "updatedAt": "2026-06-21T12:00:00Z", "isDraft": False, "repository": {"nameWithOwner": "acme/b"}},
        ]
        cache = sketchybar_prs.build_cache(mine, reviews, now=datetime(2026, 6, 23, 12, 0, tzinfo=timezone.utc))
        self.assertEqual(cache["count"], 2)
        self.assertEqual([p["id"] for p in cache["mine"]], ["acme/a#1"])
        self.assertEqual([p["id"] for p in cache["reviewRequests"]], ["acme/b#2"])

    def test_build_cache_collapses_branch_chains_to_base_and_ignores_graphite(self):
        mine = [
            {"number": 1, "title": "Bottom", "url": "https://github.com/acme/a/pull/1", "body": "Graphite PR: https://app.graphite.com/github/pr/acme/a/1", "createdAt": "2026-06-22T12:00:00Z", "updatedAt": "2026-06-22T12:00:00Z", "isDraft": False, "headRefName": "bottom", "baseRefName": "main", "repository": {"nameWithOwner": "acme/a"}},
            {"number": 2, "title": "Top", "url": "https://github.com/acme/a/pull/2", "body": "Graphite PR: https://app.graphite.com/github/pr/acme/a/2", "createdAt": "2026-06-22T12:00:00Z", "updatedAt": "2026-06-22T12:00:00Z", "isDraft": False, "headRefName": "top", "baseRefName": "bottom", "repository": {"nameWithOwner": "acme/a"}},
            {"number": 3, "title": "Single", "url": "https://github.com/acme/a/pull/3", "body": "Graphite PR: https://app.graphite.com/github/pr/acme/a/3", "createdAt": "2026-06-22T12:00:00Z", "updatedAt": "2026-06-22T12:00:00Z", "isDraft": False, "headRefName": "single", "baseRefName": "main", "repository": {"nameWithOwner": "acme/a"}},
        ]
        cache = sketchybar_prs.build_cache(mine, [], now=datetime(2026, 6, 23, 12, 0, tzinfo=timezone.utc))
        self.assertEqual([pr["id"] for pr in cache["mine"]], ["acme/a#1", "acme/a#3"])
        base = cache["mine"][0]
        single = cache["mine"][1]
        self.assertTrue(base["isStack"])
        self.assertEqual(base["stackPosition"], "1/2")
        self.assertEqual(base["displayName"], "bottom")
        self.assertEqual(base["openUrl"], "https://github.com/acme/a/pull/1")
        self.assertEqual([child["id"] for child in base["children"]], ["acme/a#2"])
        self.assertEqual(base["children"][0]["stackPosition"], "2/2")
        self.assertEqual(base["children"][0]["openUrl"], "https://github.com/acme/a/pull/2")
        self.assertFalse(single["isStack"])
        self.assertEqual(single["children"], [])

    def test_gh_stack_view_metadata_maps_pr_urls_to_stack_positions(self):
        view = {
            "branches": [
                {"name": "bottom", "pr": {"number": 10, "url": "https://github.com/acme/a/pull/10", "state": "OPEN"}},
                {"name": "top", "pr": {"number": 11, "url": "https://github.com/acme/a/pull/11", "state": "OPEN"}},
            ]
        }
        meta = sketchybar_prs.gh_stack_metadata_from_view(view, {"acme/a#10", "acme/a#11"})
        self.assertEqual(meta["acme/a#10"], {"position": "1/2", "rootId": "acme/a#10"})
        self.assertEqual(meta["acme/a#11"], {"position": "2/2", "rootId": "acme/a#10"})

    def test_load_cache_reports_missing_cache_as_stale_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = sketchybar_prs.load_cache(Path(tmp) / "missing.json")
        self.assertEqual(cache["count"], 0)
        self.assertTrue(cache["stale"])
        self.assertIn("No PR cache", cache["error"])

    def test_save_and_load_cache_round_trip(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cache.json"
            expected = {"count": 1, "mine": [], "reviewRequests": [], "error": None, "stale": False}
            sketchybar_prs.save_cache(path, expected)
            self.assertEqual(json.loads(path.read_text()), expected)
            self.assertEqual(sketchybar_prs.load_cache(path), expected)

    def test_enrich_many_preserves_order_and_calls_enricher_for_each_pr(self):
        calls = []

        def fake_enrich(raw):
            calls.append(raw["number"])
            enriched = dict(raw)
            enriched["title"] = raw["title"] + " enriched"
            return enriched

        raw = [{"number": 1, "title": "One"}, {"number": 2, "title": "Two"}]
        enriched = sketchybar_prs.enrich_many(raw, enricher=fake_enrich, max_workers=2)
        self.assertEqual([item["number"] for item in enriched], [1, 2])
        self.assertEqual([item["title"] for item in enriched], ["One enriched", "Two enriched"])
        self.assertEqual(sorted(calls), [1, 2])

    def test_render_summary_and_tsv_escape_values_for_lua_consumption(self):
        cache = {
            "count": 2,
            "stale": True,
            "error": None,
            "mine": [{"id": "acme/widgets#42", "repo": "acme/widgets", "displayName": "base", "age": "3d", "status": "open", "title": "Add\twidget\nnow", "openUrl": "https://github.com/acme/widgets/pull/42", "isStack": True, "stackPosition": "1/2", "children": [
                {"id": "acme/widgets#43", "displayName": "child", "age": "2d", "status": "ready", "title": "Child", "openUrl": "https://github.com/acme/widgets/pull/43", "isStack": True, "stackPosition": "2/2"}
            ]}],
            "reviewRequests": [],
        }
        self.assertEqual(sketchybar_prs.render_summary(cache), "2\tstale")
        self.assertEqual(
            sketchybar_prs.render_tsv(cache),
            "mine\tbase\t3d\topen\tAdd widget now\thttps://github.com/acme/widgets/pull/42\ttrue\t1/2\tbase\tacme/widgets#42\t\n"
            "mine\tchild\t2d\tready\tChild\thttps://github.com/acme/widgets/pull/43\ttrue\t2/2\tchild\tacme/widgets#43\tacme/widgets#42",
        )


if __name__ == "__main__":
    unittest.main()
