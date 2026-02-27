use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

pub const SECTOR_SIZE: usize = 512;

const CMD_READ_SECTORS: u8 = 0x20;
const CMD_WRITE_SECTORS: u8 = 0x30;
const CMD_IDENTIFY: u8 = 0xEC;

const STATUS_BSY: u8 = 0x80;
const STATUS_DRQ: u8 = 0x08;
const STATUS_ERR: u8 = 0x01;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Bus {
    Primary = 0x1F0,
    Secondary = 0x170,
}

pub struct AtaDrive {
    data_port: Port<u16>,
    error_port: PortReadOnly<u8>,
    sector_count_port: Port<u8>,
    lba_low_port: Port<u8>,
    lba_mid_port: Port<u8>,
    lba_high_port: Port<u8>,
    drive_select_port: Port<u8>,
    command_port: PortWriteOnly<u8>,
    status_port: PortReadOnly<u8>,
    is_master: bool,
}

impl AtaDrive {
    pub fn new(bus: Bus, is_master: bool) -> Self {
        let base = bus as u16;

        Self {
            data_port: Port::new(base),
            error_port: PortReadOnly::new(base + 1),
            sector_count_port: Port::new(base + 2),
            lba_low_port: Port::new(base + 3),
            lba_mid_port: Port::new(base + 4),
            lba_high_port: Port::new(base + 5),
            drive_select_port: Port::new(base + 6),
            command_port: PortWriteOnly::new(base + 7),
            status_port: PortReadOnly::new(base + 7),
            is_master,
        }
    }

    pub fn read(&mut self, lba: u32, sectors: u8, target: &mut [u16]) -> Result<(), &'static str> {
        if target.len() != (sectors as usize * 256) {
            return Err("Buffer size does not match sector count");
        }

        self.wait_busy();

        let drive_select = if self.is_master { 0xE0 } else { 0xF0 };

        unsafe {
            self.drive_select_port
                .write(drive_select | ((lba >> 24) & 0x0F) as u8);

            self.sector_count_port.write(sectors);
            self.lba_low_port.write(lba as u8);
            self.lba_mid_port.write((lba >> 8) as u8);
            self.lba_high_port.write((lba >> 16) as u8);

            self.command_port.write(CMD_READ_SECTORS);
        }

        for i in 0..sectors {
            self.poll_status()?;
            for j in 0..256 {
                let data = unsafe { self.data_port.read() };
                target[(i as usize * 256) + j] = data;
            }
        }
        Ok(())
    }

    pub fn write(&mut self, lba: u32, sectors: u8, data: &[u16]) -> Result<(), &'static str> {
        self.wait_busy();

        let drive_select = if self.is_master { 0xE0 } else { 0xF0 };

        unsafe {
            self.drive_select_port
                .write(drive_select | ((lba >> 24) & 0x0F) as u8);
            self.sector_count_port.write(sectors);
            self.lba_low_port.write(lba as u8);
            self.lba_mid_port.write((lba >> 8) as u8);
            self.lba_high_port.write((lba >> 16) as u8);
            self.command_port.write(CMD_WRITE_SECTORS);
        }

        for i in 0..sectors {
            self.poll_status()?;
            for j in 0..256 {
                unsafe {
                    self.data_port.write(data[(i as usize * 256) + j]);
                }
            }
        }
        Ok(())
    }

    fn wait_busy(&mut self) {
        while unsafe { self.status_port.read() } & STATUS_BSY != 0 {
            core::hint::spin_loop();
        }
    }

    fn poll_status(&mut self) -> Result<(), &'static str> {
        for _ in 0..4 {
            unsafe { self.status_port.read() };
        }

        loop {
            let status = unsafe { self.status_port.read() };

            if status & STATUS_ERR != 0 {
                return Err("ATA Drive Error");
            }

            if status & STATUS_BSY == 0 && status & STATUS_DRQ != 0 {
                return Ok(());
            }
        }
    }

    pub fn identify(&mut self) -> Result<[u16; 256], &'static str> {
        self.wait_busy();

        let drive_select = if self.is_master { 0xA0 } else { 0xB0 };

        unsafe {
            self.drive_select_port.write(drive_select);
            self.sector_count_port.write(0);
            self.lba_low_port.write(0);
            self.lba_mid_port.write(0);
            self.lba_high_port.write(0);
            self.command_port.write(CMD_IDENTIFY);
        }

        let status = unsafe { self.status_port.read() };

        if status == 0 {
            return Err("Drive does not exist");
        }

        self.poll_status()?;

        let mut buffer = [0u16; 256];
        for i in 0..256 {
            buffer[i] = unsafe { self.data_port.read() };
        }

        Ok(buffer)
    }

    pub fn get_total_sectors(&mut self) -> Result<u32, &'static str> {
        let data = self.identify()?;
        // Words 60-61: total user-addressable sectors (LBA28)
        let sectors = (data[60] as u32) | ((data[61] as u32) << 16);
        Ok(sectors)
    }
}
