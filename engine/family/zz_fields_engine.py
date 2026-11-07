# Fields the Paper layer reads that a Horizon engine without HorizonsX's region threads doesn't have (this
# repository's engine is Horizon's own). `region_worker`: a region thread's game on HorizonsX (paper_levels.rs's
# resident() skips its chunk wait there); never one here.
CLIENT = 'horizon-client/src/'

EDITS = [
    (CLIENT + 'game.rs', '    res: Arc<Resources>,\n', '''    res: Arc<Resources>,
    /// Always false here: no region threads. The Paper layer (shared with HorizonsX) reads it.
    region_worker: bool,
'''),
    (CLIENT + 'game.rs', '            res: res.clone(),\n', '''            res: res.clone(),
            region_worker: false,
'''),
]
