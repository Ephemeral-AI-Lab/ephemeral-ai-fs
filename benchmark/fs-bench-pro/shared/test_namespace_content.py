"""Content choice must preserve defaults and fail closed across incompatible cases."""
import unittest

import runner


class NamespaceContentTests(unittest.TestCase):
    def test_selection_and_incompatible_modes(self):
        def select(*flags, family="init_namespace", case="namespace-100-compact-v3"):
            argv = ["--family", family, *(["--case", case] if case else []), *flags]
            args = runner.build_parser().parse_args(argv)
            runner.normalize_namespace_content(args)
            runner.normalize_namespace_content(args)  # Resolution is reused for setup/proof.
            return args.case

        base = "namespace-100-compact-v3"
        self.assertEqual(select(), base)
        self.assertEqual(select("--pseudorandom"), base)
        self.assertEqual(select("--no-pseudorandom"), base + "-text-v1")
        self.assertEqual(select(case=base + "-text-v1"), base + "-text-v1")
        for options in ({"case": None}, {"family": "store_footprint"}):
            with self.assertRaises(ValueError):
                select("--no-pseudorandom", **options)
        with self.assertRaises(ValueError):
            select("--pseudorandom", case=base + "-text-v1")


if __name__ == "__main__":
    unittest.main()
