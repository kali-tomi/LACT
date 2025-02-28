use crate::app::page_section::PageSection;
use gtk::glib::{self, Object};
use lact_client::schema::{DeviceStats, PowerStats};

glib::wrapper! {
    pub struct GpuStatsSection(ObjectSubclass<imp::GpuStatsSection>)
        @extends PageSection, gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl GpuStatsSection {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        let vram_usage =
            if let (Some(used_vram), Some(total_vram)) = (stats.vram.used, stats.vram.total) {
                used_vram as f64 / total_vram as f64
            } else {
                0.0
            };
        self.set_vram_usage(vram_usage);
        self.set_vram_usage_text(format!(
            "{}/{} MiB",
            stats.vram.used.unwrap_or(0) / 1024 / 1024,
            stats.vram.total.unwrap_or(0) / 1024 / 1024,
        ));

        let clockspeed = stats.clockspeed;
        self.set_core_clock(format_clockspeed(clockspeed.gpu_clockspeed));
        self.set_current_core_clock(format_current_gfxclk(clockspeed.current_gfxclk));
        self.set_vram_clock(format_clockspeed(clockspeed.vram_clockspeed));

        let voltage = format!("{}V", stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64);
        self.set_voltage(voltage);

        let temperature = stats
            .temps
            .get("junction")
            .or_else(|| stats.temps.get("edge"))
            .and_then(|temp| temp.current)
            .unwrap_or(0.0);
        self.set_temperature(format!("{temperature}°C"));

        self.set_gpu_usage(format!("{}%", stats.busy_percent.unwrap_or(0)));

        let PowerStats {
            average: power_average,
            current: power_current,
            cap_current: power_cap_current,
            ..
        } = stats.power;

        let power_current = power_current
            .filter(|value| *value != 0.0)
            .or(power_average);

        self.set_power_usage(format!(
            "<b>{}/{}W</b>",
            power_current.unwrap_or(0.0),
            power_cap_current.unwrap_or(0.0)
        ));
    }
}

impl Default for GpuStatsSection {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use crate::app::{info_row::InfoRow, page_section::PageSection};
    use gtk::{
        glib::{self, subclass::InitializingObject, types::StaticTypeExt, Properties},
        prelude::ObjectExt,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::GpuStatsSection)]
    #[template(file = "ui/oc_page/gpu_stats_section.blp")]
    pub struct GpuStatsSection {
        #[property(get, set)]
        core_clock: RefCell<String>,
        #[property(get, set)]
        current_core_clock: RefCell<String>,
        #[property(get, set)]
        vram_clock: RefCell<String>,
        #[property(get, set)]
        voltage: RefCell<String>,
        #[property(get, set)]
        temperature: RefCell<String>,
        #[property(get, set)]
        gpu_usage: RefCell<String>,
        #[property(get, set)]
        power_usage: RefCell<String>,
        #[property(get, set)]
        vram_usage: RefCell<f64>,
        #[property(get, set)]
        vram_usage_text: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GpuStatsSection {
        const NAME: &'static str = "GpuStatsSection";
        type Type = super::GpuStatsSection;
        type ParentType = PageSection;

        fn class_init(class: &mut Self::Class) {
            InfoRow::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for GpuStatsSection {}

    impl WidgetImpl for GpuStatsSection {}
    impl BoxImpl for GpuStatsSection {}
}

fn format_clockspeed(value: Option<u64>) -> String {
    format!("{}MHz", value.unwrap_or(0))
}

fn format_current_gfxclk(value: Option<u16>) -> String {
    if let Some(v) = value {
        // if the APU/GPU dose not acually support current_gfxclk,
        // the value will be `u16::MAX (65535)`
        if v == u16::MAX {
            "N/A".to_string()
        } else {
            format!("{v}MHz")
        }
    } else {
        "N/A".to_string()
    }
}
