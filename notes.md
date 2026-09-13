Reduce graticule segments — adaptive based on step size, e.g. step * 6 segments instead of fixed 360
Cache trig values in projections — precompute sin/cos of constants (pole lat, standard parallels) once
Fast trig approximations — for graticule/tissot rendering where precision doesn't matter much
Drop the svg crate path builder — use direct string formatting (write!) instead of the ownership-heavy builder pattern
