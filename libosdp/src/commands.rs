//
// Copyright (c) 2023-2024 Siddharth Chandrasekaran <sidcha.dev@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! CP interacts with and controls PDs by sending commands to it. These commands
//! are specified by OSDP specification. This module is responsible to handling
//! such commands though [`OsdpCommand`].

use crate::OsdpError;
use crate::OsdpStatusReport;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::ConvertEndian;

type Result<T> = core::result::Result<T, OsdpError>;

/// LED Colors as specified in OSDP for the on_color/off_color parameters.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OsdpLedColor {
    /// No Color
    #[default]
    None,

    /// Red Color
    Red,

    /// Green Color
    Green,

    /// Amber Color
    Amber,

    /// Blue Color
    Blue,

    /// Magenta Color
    Magenta,

    /// Cyan Color
    Cyan,

    /// Unknown/Unsupported Color
    Unknown(u8),
}

impl From<u8> for OsdpLedColor {
    fn from(value: u8) -> Self {
        match value as libosdp_sys::osdp_led_color_e {
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_NONE => OsdpLedColor::None,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_RED => OsdpLedColor::Red,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_GREEN => OsdpLedColor::Green,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_AMBER => OsdpLedColor::Amber,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_BLUE => OsdpLedColor::Blue,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_MAGENTA => OsdpLedColor::Magenta,
            libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_CYAN => OsdpLedColor::Cyan,
            cc => OsdpLedColor::Unknown(cc),
        }
    }
}

impl From<OsdpLedColor> for u8 {
    fn from(value: OsdpLedColor) -> Self {
        match value {
            OsdpLedColor::None => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_NONE as u8,
            OsdpLedColor::Red => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_RED as u8,
            OsdpLedColor::Green => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_GREEN as u8,
            OsdpLedColor::Amber => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_AMBER as u8,
            OsdpLedColor::Blue => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_BLUE as u8,
            OsdpLedColor::Magenta => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_MAGENTA as u8,
            OsdpLedColor::Cyan => libosdp_sys::osdp_led_color_e_OSDP_LED_COLOR_CYAN as u8,
            OsdpLedColor::Unknown(cc) => cc,
        }
    }
}

/// LED params sub-structure. Part of LED command: OsdpCommandLed
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpLedParams {
    /// Control code serves different purposes based on which member of
    /// [`OsdpCommandLed`] it is used with. They are,
    ///
    /// temporary:
    ///
    /// 0 - NOP - do not alter this LED's temporary settings
    /// 1 - Cancel any temporary operation and display this LED's permanent state immediately
    /// 2 - Set the temporary state as given and start timer immediately
    ///
    /// permanent:
    ///
    /// 0 - NOP - do not alter this LED's permanent settings
    /// 1 - Set the permanent state as given
    pub control_code: u8,

    /// The ON duration of the flash, in units of 100 ms
    pub on_count: u8,

    /// The OFF duration of the flash, in units of 100 ms
    pub off_count: u8,

    /// Color to set during the ON timer
    pub on_color: OsdpLedColor,

    /// Color to set during the Off timer
    pub off_color: OsdpLedColor,

    /// Time in units of 100 ms (only for temporary mode)
    pub timer_count: u16,
}

impl From<libosdp_sys::osdp_cmd_led_params> for OsdpLedParams {
    fn from(value: libosdp_sys::osdp_cmd_led_params) -> Self {
        OsdpLedParams {
            control_code: value.control_code,
            on_count: value.on_count,
            off_count: value.off_count,
            on_color: value.on_color.into(),
            off_color: value.off_color.into(),
            timer_count: value.timer_count,
        }
    }
}

impl From<OsdpLedParams> for libosdp_sys::osdp_cmd_led_params {
    fn from(value: OsdpLedParams) -> Self {
        libosdp_sys::osdp_cmd_led_params {
            control_code: value.control_code,
            on_count: value.on_count,
            off_count: value.off_count,
            on_color: value.on_color.into(),
            off_color: value.off_color.into(),
            timer_count: value.timer_count,
        }
    }
}

/// Command to control the behavior of it's on-board LEDs
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandLed {
    /// Reader (another device connected to this PD) for which this command is
    /// issued for.
    ///
    /// 0 - self
    /// 1 - fist connected reader
    /// 2 - second connected reader
    /// ....
    pub reader: u8,

    /// LED number to operate on; 0 = first LED, 1 = second LED, etc.
    pub led_number: u8,

    /// Temporary LED activity descriptor. This operation is ephemeral and
    /// interrupts any on going permanent activity.
    pub temporary: OsdpLedParams,

    /// Permanent LED activity descriptor. This operation continues till another
    /// permanent activity overwrites this state.
    pub permanent: OsdpLedParams,
}

impl From<libosdp_sys::osdp_cmd_led> for OsdpCommandLed {
    fn from(value: libosdp_sys::osdp_cmd_led) -> Self {
        OsdpCommandLed {
            reader: value.reader,
            led_number: value.led_number,
            temporary: value.temporary.into(),
            permanent: value.permanent.into(),
        }
    }
}

impl From<OsdpCommandLed> for libosdp_sys::osdp_cmd_led {
    fn from(value: OsdpCommandLed) -> Self {
        libosdp_sys::osdp_cmd_led {
            reader: value.reader,
            led_number: value.led_number,
            temporary: value.temporary.into(),
            permanent: value.permanent.into(),
        }
    }
}

/// Command to control the behavior of a buzzer in the PD
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandBuzzer {
    /// Reader (another device connected to this PD) for which this command is
    /// issued for.
    ///
    /// 0 - self
    /// 1 - fist connected reader
    /// 2 - second connected reader
    /// ....
    pub reader: u8,

    /// Control code instructs the operation to perform:
    ///
    /// 0 - no tone
    /// 1 - off
    /// 2 - default tone
    /// 3+ - TBD
    pub control_code: u8,

    /// The ON duration of the flash, in units of 100 ms
    pub on_count: u8,

    /// The OFF duration of the flash, in units of 100 ms
    pub off_count: u8,

    /// The number of times to repeat the ON/OFF cycle; Setting this value to 0
    /// indicates the action is to be repeated forever.
    pub rep_count: u8,
}

impl From<libosdp_sys::osdp_cmd_buzzer> for OsdpCommandBuzzer {
    fn from(value: libosdp_sys::osdp_cmd_buzzer) -> Self {
        OsdpCommandBuzzer {
            reader: value.reader,
            control_code: value.control_code,
            on_count: value.on_count,
            off_count: value.off_count,
            rep_count: value.rep_count,
        }
    }
}

impl From<OsdpCommandBuzzer> for libosdp_sys::osdp_cmd_buzzer {
    fn from(value: OsdpCommandBuzzer) -> Self {
        libosdp_sys::osdp_cmd_buzzer {
            reader: value.reader,
            control_code: value.control_code,
            on_count: value.on_count,
            off_count: value.off_count,
            rep_count: value.rep_count,
        }
    }
}

/// Command to manipulate the on-board display unit (Can be LED, LCD, 7-Segment,
/// etc.,) on the PD.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandText {
    /// Reader (another device connected to this PD) for which this command is
    /// issued for.
    ///
    /// 0 - self
    /// 1 - fist connected reader
    /// 2 - second connected reader
    /// ....
    pub reader: u8,

    /// Control code instructs the operation to perform:
    ///
    /// 1 - permanent text, no wrap
    /// 2 - permanent text, with wrap
    /// 3 - temporary text, no wrap
    /// 4 - temporary text, with wrap
    pub control_code: u8,

    /// duration to display temporary text, in seconds
    pub temp_time: u8,

    /// row to display the first character (1 indexed)
    pub offset_row: u8,

    /// column to display the first character (1 indexed)
    pub offset_col: u8,

    /// The string to display (ASCII codes)
    pub data: Vec<u8>,
}

impl From<libosdp_sys::osdp_cmd_text> for OsdpCommandText {
    fn from(value: libosdp_sys::osdp_cmd_text) -> Self {
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        OsdpCommandText {
            reader: value.reader,
            control_code: value.control_code,
            temp_time: value.temp_time,
            offset_row: value.offset_row,
            offset_col: value.offset_col,
            data,
        }
    }
}

impl From<OsdpCommandText> for libosdp_sys::osdp_cmd_text {
    fn from(value: OsdpCommandText) -> Self {
        let mut data = [0; libosdp_sys::OSDP_CMD_TEXT_MAX_LEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_cmd_text {
            reader: value.reader,
            control_code: value.control_code,
            temp_time: value.temp_time,
            offset_row: value.offset_row,
            offset_col: value.offset_col,
            length: value.data.len() as u8,
            data,
        }
    }
}

/// Command to control digital output exposed by the PD.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandOutput {
    /// The output number this to apply this action.
    ///
    /// 0 - First Output
    /// 1 - Second Output
    /// ....
    pub output_no: u8,

    /// Control code instructs the operation to perform:
    ///
    /// 0 - NOP – do not alter this output
    /// 1 - set the permanent state to OFF, abort timed operation (if any)
    /// 2 - set the permanent state to ON, abort timed operation (if any)
    /// 3 - set the permanent state to OFF, allow timed operation to complete
    /// 4 - set the permanent state to ON, allow timed operation to complete
    /// 5 - set the temporary state to ON, resume perm state on timeout
    /// 6 - set the temporary state to OFF, resume permanent state on timeout
    pub control_code: u8,

    ///  Time in units of 100 ms
    pub timer_count: u16,
}

impl From<libosdp_sys::osdp_cmd_output> for OsdpCommandOutput {
    fn from(value: libosdp_sys::osdp_cmd_output) -> Self {
        OsdpCommandOutput {
            output_no: value.output_no,
            control_code: value.control_code,
            timer_count: value.timer_count,
        }
    }
}

impl From<OsdpCommandOutput> for libosdp_sys::osdp_cmd_output {
    fn from(value: OsdpCommandOutput) -> Self {
        libosdp_sys::osdp_cmd_output {
            output_no: value.output_no,
            control_code: value.control_code,
            timer_count: value.timer_count,
        }
    }
}

/// Command to set the communication parameters for the PD. The effects of this
/// command is expected to be be stored in PD's non-volatile memory as the CP
/// will expect the PD to be in this state moving forward.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpComSet {
    /// Unit ID to which this PD will respond after the change takes effect.
    pub address: u8,
    /// Baud rate.
    /// Valid values: 9600, 19200, 38400, 115200, 230400.
    pub baud_rate: u32,
}

impl From<libosdp_sys::osdp_cmd_comset> for OsdpComSet {
    fn from(value: libosdp_sys::osdp_cmd_comset) -> Self {
        OsdpComSet {
            address: value.address,
            baud_rate: value.baud_rate,
        }
    }
}

impl From<OsdpComSet> for libosdp_sys::osdp_cmd_comset {
    fn from(value: OsdpComSet) -> Self {
        libosdp_sys::osdp_cmd_comset {
            address: value.address,
            baud_rate: value.baud_rate,
        }
    }
}

/// Command to set secure channel keys to the PD.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandKeyset {
    key_type: u8,
    /// Key data
    pub data: Vec<u8>,
}

impl OsdpCommandKeyset {
    /// Create a new SCBK KeySet command for a given key
    ///
    /// # Arguments
    ///
    /// * `key` - 16 bytes of secure channel base key
    pub fn new_scbk(key: [u8; 16]) -> Self {
        let data = key.to_vec();
        Self { key_type: 1, data }
    }
}

impl From<libosdp_sys::osdp_cmd_keyset> for OsdpCommandKeyset {
    fn from(value: libosdp_sys::osdp_cmd_keyset) -> Self {
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        OsdpCommandKeyset {
            key_type: value.type_,
            data,
        }
    }
}

impl From<OsdpCommandKeyset> for libosdp_sys::osdp_cmd_keyset {
    fn from(value: OsdpCommandKeyset) -> Self {
        let mut data = [0; libosdp_sys::OSDP_CMD_KEYSET_KEY_MAX_LEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_cmd_keyset {
            type_: value.key_type,
            length: value.data.len() as u8,
            data,
        }
    }
}

/// Command to to act as a wrapper for manufacturer specific commands
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandMfg {
    /// 3-byte IEEE assigned OUI used as vendor code
    pub vendor_code: (u8, u8, u8),

    /// Command data (if any)
    pub data: Vec<u8>,
}

impl From<libosdp_sys::osdp_cmd_mfg> for OsdpCommandMfg {
    fn from(value: libosdp_sys::osdp_cmd_mfg) -> Self {
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        let bytes = value.vendor_code.to_le_bytes();
        let vendor_code: (u8, u8, u8) = (bytes[0], bytes[1], bytes[2]);
        OsdpCommandMfg { vendor_code, data }
    }
}

impl From<OsdpCommandMfg> for libosdp_sys::osdp_cmd_mfg {
    fn from(value: OsdpCommandMfg) -> Self {
        let mut data = [0; libosdp_sys::OSDP_CMD_MFG_MAX_DATALEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_cmd_mfg {
            vendor_code: value.vendor_code.as_le(),
            length: value.data.len() as u8,
            data,
        }
    }
}

/// File transfer command flag used to cancel ongoing transfers (not sent on OSDP channel).
pub const OSDP_CMD_FILE_TX_FLAG_CANCEL: u32 = 1 << 31;

/// Command to kick-off a file transfer to the PD.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCommandFileTx {
    /// Pre-agreed file ID between CP and PD
    pub id: i32,
    /// Reserved and set to zero by OSDP spec.
    /// Note that the upper bits are used by libosdp internally (IOW, not sent
    /// over the OSDP bus). Currently the following flags are defined:
    /// - OSDP_CMD_FILE_TX_FLAG_CANCEL
    pub flags: u32,
}

impl From<libosdp_sys::osdp_cmd_file_tx> for OsdpCommandFileTx {
    fn from(value: libosdp_sys::osdp_cmd_file_tx) -> Self {
        OsdpCommandFileTx {
            id: value.id,
            flags: value.flags,
        }
    }
}

impl From<OsdpCommandFileTx> for libosdp_sys::osdp_cmd_file_tx {
    fn from(value: OsdpCommandFileTx) -> Self {
        libosdp_sys::osdp_cmd_file_tx {
            id: value.id,
            flags: value.flags,
        }
    }
}

/// Extended READ/WRITE Command Mode 0 - Mode Set
///
/// Set and configure the background behavior mode.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCmdSetModeData {
    /// Background operation mode code
    pub mode_code: u8,
    /// Mode configuration
    pub mode_config: u8,
}

impl From<libosdp_sys::osdp_xwr_mode_set> for OsdpCmdSetModeData {
    fn from(value: libosdp_sys::osdp_xwr_mode_set) -> Self {
        Self {
            mode_code: value.mode_code,
            mode_config: value.mode_config,
        }
    }
}

impl From<OsdpCmdSetModeData> for libosdp_sys::osdp_xwr_mode_set {
    fn from(value: OsdpCmdSetModeData) -> Self {
        Self {
            mode_code: value.mode_code,
            mode_config: value.mode_config,
        }
    }
}

/// Extended READ/WRITE Command Mode 1 - Transparent Content Send Request Data
///
/// The embedded APDU shall be passed to the specified reader.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCmdTransparentSendData {
    /// Reader number
    pub reader: u8,
    /// Valid APDU to send to the smart card
    pub apdu: Vec<u8>,
}

impl From<libosdp_sys::osdp_xwr_transp_send> for OsdpCmdTransparentSendData {
    fn from(value: libosdp_sys::osdp_xwr_transp_send) -> Self {
        let n = value.length as usize;
        let apdu = value.apdu[0..n].to_vec();
        Self {
            reader: value.reader,
            apdu,
        }
    }
}

impl From<OsdpCmdTransparentSendData> for libosdp_sys::osdp_xwr_transp_send {
    fn from(value: OsdpCmdTransparentSendData) -> Self {
        let mut apdu = [0; libosdp_sys::OSDP_CMD_XWR_APDU_MAX_LEN as usize];
        apdu[..value.apdu.len()].copy_from_slice(&value.apdu[..]);
        Self {
            reader: value.reader,
            length: value.apdu.len() as u8,
            apdu,
        }
    }
}

/// Extended READ/WRITE Command Mode 1 - Connection Done
///
/// Instruct the PD (reader) to disconnect from the smart card.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCmdCardScanData {
    /// Reader number
    pub reader: u8,
}

impl From<libosdp_sys::osdp_xwr_sc_scan> for OsdpCmdCardScanData {
    fn from(value: libosdp_sys::osdp_xwr_sc_scan) -> Self {
        Self {
            reader: value.reader,
        }
    }
}

impl From<OsdpCmdCardScanData> for libosdp_sys::osdp_xwr_sc_scan {
    fn from(value: OsdpCmdCardScanData) -> Self {
        Self {
            reader: value.reader,
        }
    }
}

/// Extended READ/WRITE Mode 1 - Smart Card Scan
///
/// Identify if a smart card is present at the reader.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCmdCardDisconnectData {
    /// Reader number
    pub reader: u8,
}

impl From<libosdp_sys::osdp_xwr_sc_disconnect> for OsdpCmdCardDisconnectData {
    fn from(value: libosdp_sys::osdp_xwr_sc_disconnect) -> Self {
        Self {
            reader: value.reader,
        }
    }
}

impl From<OsdpCmdCardDisconnectData> for libosdp_sys::osdp_xwr_sc_disconnect {
    fn from(value: OsdpCmdCardDisconnectData) -> Self {
        Self {
            reader: value.reader,
        }
    }
}

/// Extended READ/WRITE Command Mode-01 - Request Secure PIN Entry
///
/// Instruct the PD (reader) to perform a local Secure PIN Entry (SPE) sequence
/// with the smart card. It also includes an APDU for the smart card.
/// When the reader receives this packet, it autonomously prompts the user for
/// their PIN, inserts the PIN into the APDU and sends it to the smart card.
/// The reader should restore the display to its previous state when done
/// processing the user input.
/// While processing this message, the reader should not add any keys to the
/// keypad buffer.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OsdpCmdSecurePinData {
    /// Reader number
    pub reader: u8,
    /// Timeout in seconds (0x00 means use default timeout)
    pub timeout: u8,
    /// Timeout in seconds after first key stroke
    pub timeout2: u8,
    /// Formatting USB_CCID_PIN_FORMAT_xxx
    pub format_string: u8,
    /// PIN block string
    ///
    /// Bits 3-0 - PIN block size in bytes after justification and formatting.
    /// Bits 7-4 - Bit size of PIN length in APDU.
    pub pin_block_string: u8,
    /// Bit length format
    ///
    /// Bits 3-0 - PIN length position in system units
    /// Bits 7-5 - Reserved for future use, bit 4 set if system units are bytes
    ///            clear if system units are bits.
    pub pin_len_format: u8,
    /// PIN maximum extra digit
    ///
    /// XXYY, where XX is minimum PIN size in digits, YY is maximum.
    pub pin_max_extra_digit: u16,
    /// Conditions under which PIN entry should be considered complete
    pub entry_validation_condition: u8,
    /// Number of verification messages to display for PIN
    pub number_message: u8,
    /// Language for messages
    pub language_id: u16,
    /// Message index
    pub msg_index: u8,
    /// T=1 I-block prologue field to use (fill with 0x00)
    pub teo_prologue: u32,
    /// APDU data to send to the smart card
    pub apdu: Vec<u8>,
}

impl From<libosdp_sys::osdp_xwr_secure_pin> for OsdpCmdSecurePinData {
    fn from(value: libosdp_sys::osdp_xwr_secure_pin) -> Self {
        let n = value.apdu_length as usize;
        let apdu = value.apdu[0..n].to_vec();
        Self {
            reader: value.reader,
            timeout: value.timeout,
            timeout2: value.timeout2,
            format_string: value.format_string,
            pin_block_string: value.pin_block_string,
            pin_len_format: value.pin_len_format,
            pin_max_extra_digit: value.pin_max_extra_digit,
            entry_validation_condition: value.entry_validation_condition,
            number_message: value.number_message,
            language_id: value.language_id,
            msg_index: value.msg_index,
            teo_prologue: value.teo_prologue,
            apdu,
        }
    }
}

impl From<OsdpCmdSecurePinData> for libosdp_sys::osdp_xwr_secure_pin {
    fn from(value: OsdpCmdSecurePinData) -> Self {
        let mut apdu = [0; libosdp_sys::OSDP_CMD_XWR_APDU_MAX_LEN as usize];
        apdu[..value.apdu.len()].copy_from_slice(&value.apdu[..]);
        Self {
            reader: value.reader,
            timeout: value.timeout,
            timeout2: value.timeout2,
            format_string: value.format_string,
            pin_block_string: value.pin_block_string,
            pin_len_format: value.pin_len_format,
            pin_max_extra_digit: value.pin_max_extra_digit,
            entry_validation_condition: value.entry_validation_condition,
            number_message: value.number_message,
            language_id: value.language_id,
            msg_index: value.msg_index,
            teo_prologue: value.teo_prologue,
            apdu_length: value.apdu.len() as u32,
            apdu,
        }
    }
}

/// Extended write command types
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OsdpCmdXWriteType {
    /// Returns the current mode in effect.
    GetMode,
    /// En-/Disable the specified mode.
    SetMode,
    /// Pass the embedded APDU to the specified reader.
    TransparentSend,
    /// Instruct the designated reader to perform a smart card scan.
    CardScan,
    /// Notifies the designated reader to terminate its connection to the smart card.
    CardDisconnect,
    /// Instruct the designated reader to perform a "Secure PIN Entry".
    SecurePin,
}
impl OsdpCmdXWriteType {
    /// Convert extended write mode and command an enumerated type.
    pub fn try_from_mode_and_command(mode: u8, command: u8) -> Option<Self> {
        match (mode, command) {
            (0, 1) => Some(Self::GetMode),
            (0, 2) => Some(Self::SetMode),
            (1, 1) => Some(Self::TransparentSend),
            (1, 2) => Some(Self::CardDisconnect),
            (1, 3) => Some(Self::SecurePin),
            (1, 4) => Some(Self::CardScan),
            (_, _) => None,
        }
    }

    /// Get extended write mode and command from enumerated type.
    pub fn to_mode_and_command(self) -> (u8, u8) {
        match self {
            Self::GetMode => (0, 1),
            Self::SetMode => (0, 2),
            Self::TransparentSend => (1, 1),
            Self::CardDisconnect => (1, 2),
            Self::SecurePin => (1, 3),
            Self::CardScan => (1, 4),
        }
    }
}

impl From<&OsdpCommandXWrite> for OsdpCmdXWriteType {
    fn from(value: &OsdpCommandXWrite) -> Self {
        match value {
            OsdpCommandXWrite::GetMode(_) => Self::GetMode,
            OsdpCommandXWrite::SetMode(_) => Self::SetMode,
            OsdpCommandXWrite::TransparentSend(_) => Self::TransparentSend,
            OsdpCommandXWrite::CardScan(_) => Self::CardScan,
            OsdpCommandXWrite::CardDisconnect(_) => Self::CardDisconnect,
            OsdpCommandXWrite::SecurePin(_) => Self::SecurePin,
        }
    }
}

/// Command to facilitate communications with an ISO 7816-4 based credential.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OsdpCommandXWrite {
    /// Returns the current mode in effect.
    GetMode(()),
    /// En-/Disable the specified mode.
    SetMode(OsdpCmdSetModeData),
    /// Pass the embedded APDU to the specified reader.
    TransparentSend(OsdpCmdTransparentSendData),
    /// Instruct the designated reader to perform a smart card scan.
    CardScan(OsdpCmdCardScanData),
    /// Notifies the designated reader to terminate its connection to the smart card.
    CardDisconnect(OsdpCmdCardDisconnectData),
    /// Instruct the designated reader to perform a "Secure PIN Entry".
    SecurePin(OsdpCmdSecurePinData),
}

impl From<OsdpCommandXWrite> for libosdp_sys::osdp_cmd_xwrite {
    fn from(value: OsdpCommandXWrite) -> Self {
        let (mode, command) = OsdpCmdXWriteType::from(&value).to_mode_and_command();
        match value {
            OsdpCommandXWrite::GetMode(_) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    mode_set: OsdpCmdSetModeData::default().into(),
                },
            },
            OsdpCommandXWrite::SetMode(c) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    mode_set: c.clone().into(),
                },
            },
            OsdpCommandXWrite::TransparentSend(c) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    transp_send: c.clone().into(),
                },
            },
            OsdpCommandXWrite::CardDisconnect(c) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    sc_disco: c.clone().into(),
                },
            },
            OsdpCommandXWrite::SecurePin(c) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    secure_pin: c.clone().into(),
                },
            },
            OsdpCommandXWrite::CardScan(c) => Self {
                mode,
                command,
                __bindgen_anon_1: libosdp_sys::osdp_cmd_xwrite__bindgen_ty_1 {
                    sc_scan: c.clone().into(),
                },
            },
        }
    }
}

impl TryFrom<libosdp_sys::osdp_cmd_xwrite> for OsdpCommandXWrite {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_cmd_xwrite) -> Result<Self> {
        OsdpCmdXWriteType::try_from_mode_and_command(value.mode, value.command)
            .map(|t| match t {
                OsdpCmdXWriteType::GetMode => Self::GetMode(()),
                OsdpCmdXWriteType::SetMode => {
                    Self::SetMode(unsafe { value.__bindgen_anon_1.mode_set.into() })
                }
                OsdpCmdXWriteType::TransparentSend => {
                    Self::TransparentSend(unsafe { value.__bindgen_anon_1.transp_send.into() })
                }
                OsdpCmdXWriteType::CardScan => {
                    Self::CardScan(unsafe { value.__bindgen_anon_1.sc_scan.into() })
                }
                OsdpCmdXWriteType::CardDisconnect => {
                    Self::CardDisconnect(unsafe { value.__bindgen_anon_1.sc_disco.into() })
                }
                OsdpCmdXWriteType::SecurePin => {
                    Self::SecurePin(unsafe { value.__bindgen_anon_1.secure_pin.into() })
                }
            })
            .ok_or(OsdpError::Parse(
                "Unknown extended write mode and command combination".into(),
            ))
    }
}

/// CP interacts with and controls PDs by sending commands to it. The commands
/// in this enum are specified by OSDP specification.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OsdpCommand {
    /// Command to control the behavior of it’s on-board LEDs
    Led(OsdpCommandLed),

    /// Command to control the behavior of a buzzer in the PD
    Buzzer(OsdpCommandBuzzer),

    /// Command to manipulate the on-board display unit (Can be LED, LCD,
    /// 7-Segment, etc.,) on the PD
    Text(OsdpCommandText),

    /// Command to control digital output exposed by the PD
    Output(OsdpCommandOutput),

    /// Command to request setting the communication parameters for the PD.
    ComSet(OsdpComSet),

    /// Set communication parameter completed.
    /// The effects of this command is expected to be be stored in PD’s non-volatile
    /// memory as the CP will expect the PD to be in this state moving forward
    ComSetDone(OsdpComSet),

    /// Command to set secure channel keys to the PD
    KeySet(OsdpCommandKeyset),

    /// Command to to act as a wrapper for manufacturer specific commands
    Mfg(OsdpCommandMfg),

    /// Command to kick-off a file transfer to the PD
    FileTx(OsdpCommandFileTx),

    /// Command to query status from the PD
    Status(OsdpStatusReport),

    /// Command to facilitate communications with an ISO 7816-4 based credential
    ExtendedWrite(OsdpCommandXWrite),
}

impl From<OsdpCommand> for libosdp_sys::osdp_cmd {
    fn from(value: OsdpCommand) -> Self {
        match value {
            OsdpCommand::Led(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_LED,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 {
                    led: c.clone().into(),
                },
            },
            OsdpCommand::Buzzer(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_BUZZER,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { buzzer: c.into() },
            },
            OsdpCommand::Text(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_TEXT,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 {
                    text: c.clone().into(),
                },
            },
            OsdpCommand::Output(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_OUTPUT,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { output: c.into() },
            },
            OsdpCommand::ComSet(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_COMSET,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { comset: c.into() },
            },
            OsdpCommand::ComSetDone(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_COMSET_DONE,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { comset: c.into() },
            },
            OsdpCommand::KeySet(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_KEYSET,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 {
                    keyset: c.clone().into(),
                },
            },
            OsdpCommand::Mfg(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_MFG,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 {
                    mfg: c.clone().into(),
                },
            },
            OsdpCommand::FileTx(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_FILE_TX,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { file_tx: c.into() },
            },
            OsdpCommand::Status(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_STATUS,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { status: c.into() },
            },
            OsdpCommand::ExtendedWrite(c) => libosdp_sys::osdp_cmd {
                _node: unsafe { core::mem::zeroed() },
                id: libosdp_sys::osdp_cmd_e_OSDP_CMD_XWRITE,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_cmd__bindgen_ty_1 { xwrite: c.into() },
            },
        }
    }
}

impl TryFrom<libosdp_sys::osdp_cmd> for OsdpCommand {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_cmd) -> Result<Self> {
        match value.id {
            libosdp_sys::osdp_cmd_e_OSDP_CMD_LED => Ok(OsdpCommand::Led(unsafe {
                value.__bindgen_anon_1.led.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_BUZZER => Ok(OsdpCommand::Buzzer(unsafe {
                value.__bindgen_anon_1.buzzer.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_TEXT => Ok(OsdpCommand::Text(unsafe {
                value.__bindgen_anon_1.text.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_OUTPUT => Ok(OsdpCommand::Output(unsafe {
                value.__bindgen_anon_1.output.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_COMSET => Ok(OsdpCommand::ComSet(unsafe {
                value.__bindgen_anon_1.comset.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_COMSET_DONE => Ok(OsdpCommand::ComSetDone(unsafe {
                value.__bindgen_anon_1.comset.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_KEYSET => Ok(OsdpCommand::KeySet(unsafe {
                value.__bindgen_anon_1.keyset.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_MFG => Ok(OsdpCommand::Mfg(unsafe {
                value.__bindgen_anon_1.mfg.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_FILE_TX => Ok(OsdpCommand::FileTx(unsafe {
                value.__bindgen_anon_1.file_tx.into()
            })),
            libosdp_sys::osdp_cmd_e_OSDP_CMD_STATUS => {
                let data = unsafe { value.__bindgen_anon_1.status.try_into() }?;
                Ok(OsdpCommand::Status(data))
            }
            libosdp_sys::osdp_cmd_e_OSDP_CMD_XWRITE => {
                let data = unsafe { value.__bindgen_anon_1.xwrite.try_into() }?;
                Ok(OsdpCommand::ExtendedWrite(data))
            }
            _ => Err(OsdpError::Parse("Unknown command".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::OsdpCommandMfg;
    use libosdp_sys::osdp_cmd_mfg;

    #[test]
    fn test_command_mfg() {
        let cmd = OsdpCommandMfg {
            vendor_code: (0x05, 0x07, 0x09),
            data: vec![0x55, 0xAA],
        };
        let cmd_struct: osdp_cmd_mfg = cmd.clone().into();

        assert_eq!(cmd_struct.vendor_code, 0x90705);
        assert_eq!(cmd_struct.length, 2);
        assert_eq!(cmd_struct.data[0], 0x55);
        assert_eq!(cmd_struct.data[1], 0xAA);

        assert_eq!(cmd, cmd_struct.into());
    }
}
