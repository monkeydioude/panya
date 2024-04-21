pub fn compute_refresh_avg(
    current_avg: f32,
    time_to_refresh: i64,
    refresh_count: i32,
) -> f32 {
    // multiplying by 1000 to avoid to lose f32's 3 decimals precision
    (
        ((refresh_count as i64 * (current_avg * 1000.) as i64) + (time_to_refresh * 1000))
        / ((refresh_count + 1) as i64 * 1000)
    ) as f32
}