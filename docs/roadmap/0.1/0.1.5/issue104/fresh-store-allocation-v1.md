# Fresh Store allocation observation v1

Before the first init_namespace/store_footprint family: source inspection shows
these routes emit database length/durable byte accounting, but not consistently
physical allocation including sidecars. Reuse the tested closed-Store stat helper
at work/store.sqlite for these two routes, after product/resource timing and
before cleanup. This is the same physical identity/allocation boundary as the SDK
supplement; no reads, digest, rewrite, changed fixture or timer. Neither family
has run yet. Existing completed families and SDK samples are unaffected: their
branch behavior remains identical. There is no recollection of successful work.
