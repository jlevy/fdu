# Performance guide

Measure one hypothesis at a time against a saved control binary. Run control and
candidate in interleaved pairs, verify every sample against the same content oracle,
and retain rejected experiments so the next change starts with evidence.

Cold scans, warm reconciliation, snapshot loading, and report generation are distinct
paths. Profile the path under study before changing it and record enough trials to
separate a real shift from filesystem noise.
