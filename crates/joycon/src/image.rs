use joycon_sys::mcu::{ir::Resolution, *};
use joycon_sys::*;

pub struct Image {
    buffer: Box<[[u8; 300]; 0x100]>,
    resolution: ir::Resolution,
    prev_fragment_id: u8,
    changing_resolution: bool,

    // The last COMPLETED image.
    pub last_image: Option<image::GrayImage>,
}

impl Image {
    pub fn new() -> Image {
        Image {
            buffer: Box::new([[0; 300]; 0x100]),
            resolution: Resolution::default(),
            prev_fragment_id: 0,
            changing_resolution: false,
            last_image: None,
        }
    }

    pub fn change_resolution(&mut self, resolution: ir::Resolution) {
        self.resolution = resolution;
        self.changing_resolution = true;
        self.prev_fragment_id = 0;
        self.last_image = None;
    }

    pub fn handle(&mut self, report: &MCUReport) -> [Option<OutputReport>; 2] {
        if let Some(packet) = report.ir_data() {
            let fragment = packet.frag_number;
            let max_fragment = self.resolution.max_fragment_id();

            // After changing resolution, ignore everything until
            // the controller starts a fresh image at fragment 0.
            if self.changing_resolution {
                if fragment != 0 {
                    return [Some(OutputReport::ir_ack(fragment)), None];
                }

                self.changing_resolution = false;
                self.prev_fragment_id = 0;
            }

            // Store the fragment.
            self.buffer[fragment as usize] = packet.img_fragment;

            // Detect a gap in the normal sequence.
            let resend = if fragment > 0
                && self.prev_fragment_id > 0
                && fragment > self.prev_fragment_id + 1
            {
                let missing = self.prev_fragment_id + 1;

                println!("IR: gap: expected {}, received {}", missing, fragment);

                Some(OutputReport::ir_resend(missing))
            } else {
                None
            };

            // ----------------------------------------------------
            // IMPORTANT:
            //
            // Only create last_image when the FINAL fragment
            // arrives.
            // ----------------------------------------------------

            if fragment == max_fragment {
                let (width, height) = self.resolution.size();

                let mut buffer = Vec::with_capacity((width * height) as usize);

                for fragment in self.buffer.iter().take(max_fragment as usize + 1) {
                    buffer.extend_from_slice(fragment);
                }

                let image = image::GrayImage::from_raw(width, height, buffer)
                    .expect("invalid IR image dimensions");

                self.last_image = Some(image::imageops::rotate90(&image));

                println!("IR: COMPLETE FRAME");

                self.prev_fragment_id = 0;
            } else {
                self.prev_fragment_id = fragment;
            }

            [Some(OutputReport::ir_ack(fragment)), resend]
        } else if report.id() == MCUReportId::Empty {
            [
                Some(OutputReport::ir_resend(self.prev_fragment_id + 1)),
                None,
            ]
        } else if report.id() == MCUReportId::EmptyAwaitingCmd {
            [Some(OutputReport::ir_ack(self.prev_fragment_id)), None]
        } else {
            [None, None]
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}
