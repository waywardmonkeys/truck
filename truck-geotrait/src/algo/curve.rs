use super::*;

/// Divides the domain into equal parts, examines all the values, and returns `t` such that `curve.subs(t)` is closest to `point`.
/// This method is useful to get an efficient hint of `search_nearest_parameter`.
pub fn presearch<C>(curve: &C, point: C::Point, range: (f64, f64), division: usize) -> f64
where
    C: ParametricCurve,
    C::Point: MetricSpace<Metric = f64> + Copy, {
    let (t0, t1) = range;
    let mut res = t0;
    let mut min = std::f64::INFINITY;
    for i in 0..=division {
        let p = i as f64 / division as f64;
        let t = t0 * (1.0 - p) + t1 * p;
        let dist = curve.subs(t).distance2(point);
        if dist < min {
            min = dist;
            res = t;
        }
    }
    res
}

/// Searches the nearest parameter by Newton's method.
pub fn search_nearest_parameter<C>(
    curve: &C,
    point: C::Point,
    hint: f64,
    trials: usize,
) -> Option<f64>
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    let pt = curve.subs(hint);
    let der = curve.der(hint);
    let der2 = curve.der2(hint);
    let f = der.dot(pt - point);
    let fprime = der2.dot(pt - point) + der.magnitude2();
    let dermag = f64::min(der.magnitude(), 1.0);
    if f64::abs(f) < TOLERANCE * dermag || fprime.so_small() {
        return Some(hint);
    } else if trials == 0 {
        None
    } else {
        search_nearest_parameter(curve, point, hint - f / fprime, trials - 1)
    }
}

/// Searches the parameter by Newton's method.
pub fn search_parameter<C>(curve: &C, point: C::Point, hint: f64, trials: usize) -> Option<f64>
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: InnerSpace<Scalar = f64> + Tolerance, {
    search_nearest_parameter(curve, point, hint, trials).and_then(|t| {
        match point.to_vec().near(&curve.subs(t).to_vec()) {
            true => Some(t),
            false => None,
        }
    })
}

/// Creates the curve division
pub fn parameter_division<C>(curve: &C, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<C::Point>)
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>, {
    sub_parameter_division(curve, range, (curve.subs(range.0), curve.subs(range.1)), tol)
}

fn sub_parameter_division<C>(curve: &C, range: (f64, f64), ends: (C::Point, C::Point), tol: f64) -> (Vec<f64>, Vec<C::Point>)
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>, {
    let p = 0.5 + (0.2 * rand::random::<f64>() - 0.1);
    let t = range.0 * (1.0 - p) + range.1 * p;
    let mid = ends.0 + (ends.1 - ends.0) * p;
    if curve.subs(t).distance(mid) < tol {
        (vec![range.0, range.1], vec![ends.0, ends.1])
    } else {
        let mid = (range.0 + range.1) / 2.0;
        let (mut params, mut pts) = parameter_division(curve, (range.0, mid), tol);
        let _ = (params.pop(), pts.pop());
        let (new_params, new_pts) = parameter_division(curve, (mid, range.1), tol);
        params.extend(new_params);
        pts.extend(new_pts);
        (params, pts)
    }
}
