#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use slint::SharedString;
slint::include_modules!();

struct Config {
    max_rod: f64,
    leftover: f64,
    parts: i32,
    part_len: f64,
    remainings: i32,
}

fn parse_input(ui: &AppWindow) -> Result<Config, SharedString> {
    ui.set_status_text("".into());
    let parse_i32 = |s: &str, name: &str| {
        s.trim()
            .parse::<i32>()
            .map_err(|e| SharedString::from(format!("Ошибка '{}' в поле '{}'", e, name)))
    };
    let parse_f64 = |s: &str, name: &str| {
        s.trim()
            .replace(',', ".")
            .parse::<f64>()
            .map_err(|e| SharedString::from(format!("Ошибка '{}' в поле '{}'", e, name)))
    };

    let max_rod = parse_f64(&ui.get_max_rod_length(), "Максимальная длина прута")?;
    let leftover = parse_f64(&ui.get_tail(), "Остаток")?;
    let parts = parse_i32(&ui.get_parts_count(), "Количество заготовок")?;
    let part_len = parse_f64(&ui.get_part_length(), "Длина заготовки")?;

    if max_rod <= 0.0 {
        return Err(SharedString::from(
            "Максимальная длина прута должна быть больше нуля",
        ));
    }
    if leftover < 0.0 {
        return Err(SharedString::from("Остаток не может быть отрицательным"));
    }
    if part_len <= 0.0 {
        return Err(SharedString::from(
            "Длина заготовки должна быть больше нуля",
        ));
    }
    if (max_rod - leftover) / part_len < 1.0 {
        return Err(SharedString::from(
            "Заготовка с учётом остатка не помещается в прут",
        ));
    }

    Ok(Config {
        max_rod,
        leftover,
        parts,
        part_len,
        remainings: ui.get_remainings_count(),
    })
}

fn rod_suffix(count: i32) -> &'static str {
    if count % 10 == 1 && count % 100 != 11 {
        ""
    } else if (2..=4).contains(&(count % 10)) && !(12..=14).contains(&(count % 100)) {
        "а"
    } else {
        "ов"
    }
}

fn part_suffix(count: i32) -> &'static str {
    if count % 10 == 1 && count % 100 != 11 {
        "ь"
    } else if (2..=4).contains(&(count % 10)) && !(12..=14).contains(&(count % 100)) {
        "и"
    } else {
        "ей"
    }
}

fn fmt_len(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.3}", v)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn max_per_rod(cfg: &Config) -> i32 {
    ((cfg.max_rod - cfg.leftover) / cfg.part_len).floor() as i32
}

fn formula(p: f64, o: f64, m: f64) -> SharedString {
    let avail = m - o;
    let n_max = (avail / p).floor() as i64;
    SharedString::from(format!(
        "\nПуть расчета:\n{} × n + {} ≤ {}\n{} × n ≤ {}\nN ≤ {:.3}\nМаксимум деталей на прут: n = {}\n",
        fmt_len(p),
        fmt_len(o),
        fmt_len(m),
        fmt_len(p),
        fmt_len(m - o),
        avail / p,
        n_max
    ))
}

fn calc_simple(cfg: &Config) -> SharedString {
    let max_per = max_per_rod(cfg);
    let full = cfg.parts / max_per;
    let rem = cfg.parts % max_per;
    let mut out = String::new();
    if full > 0 {
        out += &format!(
            "- {} прут{} по {} мм ({} детал{} с прута)\n",
            full,
            rod_suffix(full),
            fmt_len(max_per as f64 * cfg.part_len + cfg.leftover),
            max_per,
            part_suffix(max_per)
        );
    }
    if rem > 0 {
        out += &format!(
            "- 1 прут: {} мм ({} детал{} с прута)\n",
            fmt_len(rem as f64 * cfg.part_len + cfg.leftover),
            rem,
            part_suffix(rem)
        );
    }
    let rods_count = full + if rem > 0 { 1 } else { 0 };
    let total_mat =
        (full * max_per + rem) as f64 * cfg.part_len + rods_count as f64 * cfg.leftover;
    let total_left = rods_count as f64 * cfg.leftover;
    out += &format!(
        "Общий расход материала: {} мм\nОбщий остаток: {} мм",
        fmt_len(total_mat),
        fmt_len(total_left)
    );
    SharedString::from(out) + &formula(cfg.part_len, cfg.leftover, cfg.max_rod)
}

fn calc_all_equal(cfg: &Config) -> SharedString {
    let max_per = max_per_rod(cfg);
    let mut best = (0, i32::MAX);
    for n in 1..=max_per {
        if n as f64 * cfg.part_len + cfg.leftover <= cfg.max_rod {
            let rods = (cfg.parts + n - 1) / n;
            if rods * n == cfg.parts && rods < best.1 {
                best = (n, rods);
            }
        }
    }
    if best.0 == 0 {
        return SharedString::from("Невозможно равномерно распределить детали по прутам.");
    }
    let (n, rods) = best;
    let len = n as f64 * cfg.part_len + cfg.leftover;
    let mat = rods as f64 * len;
    let left = rods as f64 * cfg.leftover;
    SharedString::from(format!(
        "- {} прут{} по {} мм ({} детал{} с прута)\nОбщий расход материала: {} мм\nОбщий остаток: {} мм",
        rods,
        rod_suffix(rods),
        fmt_len(len),
        n,
        part_suffix(n),
        fmt_len(mat),
        fmt_len(left)
    )) + &formula(cfg.part_len, cfg.leftover, cfg.max_rod)
}

fn calc_complicated(cfg: &Config) -> SharedString {
    let max_per = max_per_rod(cfg);

    if cfg.remainings <= 0 || cfg.parts <= max_per {
        return calc_simple(cfg);
    }

    let total_rods_needed = (cfg.parts + max_per - 1) / max_per;

    if total_rods_needed <= cfg.remainings {
        let parts_per_rod = (cfg.parts + cfg.remainings - 1) / cfg.remainings;
        let rod_length = parts_per_rod as f64 * cfg.part_len + cfg.leftover;

        let total_mat = cfg.remainings as f64 * rod_length;
        let total_left = cfg.remainings as f64 * cfg.leftover;

        let out = format!(
            "- {} прут{} по {} мм ({} детал{} с прута)\nОбщий расход материала: {} мм\nОбщий остаток: {} мм",
            cfg.remainings,
            rod_suffix(cfg.remainings),
            fmt_len(rod_length),
            parts_per_rod,
            part_suffix(parts_per_rod),
            fmt_len(total_mat),
            fmt_len(total_left)
        );

        return SharedString::from(out) + &formula(cfg.part_len, cfg.leftover, cfg.max_rod);
    }

    let full_rods = total_rods_needed - cfg.remainings;

    let parts_on_full_rods = full_rods * max_per;

    if parts_on_full_rods >= cfg.parts {
        return calc_simple(cfg);
    }

    let remaining_parts = cfg.parts - parts_on_full_rods;

    let parts_per_remaining_rod = (remaining_parts + cfg.remainings - 1) / cfg.remainings;

    let mut out = String::new();

    if full_rods > 0 {
        out += &format!(
            "- {} прут{} по {} мм ({} детал{} с прута)\n",
            full_rods,
            rod_suffix(full_rods),
            fmt_len(max_per as f64 * cfg.part_len + cfg.leftover),
            max_per,
            part_suffix(max_per)
        );
    }

    let remaining_rod_length = parts_per_remaining_rod as f64 * cfg.part_len + cfg.leftover;
    out += &format!(
        "- {} прут{} по {} мм ({} детал{} с прута)\n",
        cfg.remainings,
        rod_suffix(cfg.remainings),
        fmt_len(remaining_rod_length),
        parts_per_remaining_rod,
        part_suffix(parts_per_remaining_rod)
    );

    let total_mat = full_rods as f64 * (max_per as f64 * cfg.part_len + cfg.leftover)
        + cfg.remainings as f64 * remaining_rod_length;
    let total_left = (full_rods + cfg.remainings) as f64 * cfg.leftover;

    out += &format!(
        "Общий расход материала: {} мм\nОбщий остаток: {} мм",
        fmt_len(total_mat),
        fmt_len(total_left)
    );

    SharedString::from(out) + &formula(cfg.part_len, cfg.leftover, cfg.max_rod)
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    ui.on_calculate({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            match parse_input(&ui) {
                Err(e) => {
                    ui.set_simple_result("".into());
                    ui.set_all_equal_result("".into());
                    ui.set_complicated_result("".into());
                    ui.set_status_text(e);
                }
                Ok(cfg) => {
                    ui.set_simple_result(calc_simple(&cfg));
                    ui.set_all_equal_result(calc_all_equal(&cfg));
                    ui.set_complicated_result(calc_complicated(&cfg));
                    ui.set_status_text("".into());
                }
            }
        }
    });
    ui.run()?;
    Ok(())
}
