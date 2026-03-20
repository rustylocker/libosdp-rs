//
// Copyright (c) 2023-2024 Siddharth Chandrasekaran <sidcha.dev@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! OSDP PDs have to send messages to it's controlling unit - CP to intimate it
//! about various events that originate there (such as key press, card reads,
//! etc.,). They do this by creating an "event" and sending it to the CP. This
//! module is responsible to handling such events though [`OsdpEvent`].

use crate::OsdpError;
use alloc::{vec::Vec, format};
use serde::{Deserialize, Serialize};

use super::ConvertEndian;

type Result<T> = core::result::Result<T, OsdpError>;

#[cfg(feature = "defmt")]
use defmt::panic;

/// Various card formats that a PD can support. This is sent to CP when a PD
/// must report a card read
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OsdpCardFormats {
    /// Card format is not specified
    #[default]
    Unspecified,

    /// Wiegand format
    Wiegand,
}

impl TryFrom<libosdp_sys::osdp_event_cardread_format_e> for OsdpCardFormats {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_event_cardread_format_e) -> Result<Self> {
        match value {
            libosdp_sys::osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_UNSPECIFIED => {
                Ok(OsdpCardFormats::Unspecified)
            }
            libosdp_sys::osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_WIEGAND => {
                Ok(OsdpCardFormats::Wiegand)
            }
            cf => Err(OsdpError::Parse(format!("Unknown card format ({cf})").into())),
        }
    }
}

impl From<OsdpCardFormats> for libosdp_sys::osdp_event_cardread_format_e {
    fn from(val: OsdpCardFormats) -> Self {
        match val {
            OsdpCardFormats::Unspecified => {
                libosdp_sys::osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_UNSPECIFIED
            }
            OsdpCardFormats::Wiegand => {
                libosdp_sys::osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_WIEGAND
            }
        }
    }
}

/// Event that describes card read activity on the PD
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventCardRead {
    /// Reader (another device connected to this PD) which caused this event
    ///
    /// 0 - self
    /// 1 - fist connected reader
    /// 2 - second connected reader
    /// ....
    pub reader_no: i32,

    /// Format of the card that was read
    pub format: OsdpCardFormats,

    /// The direction of the PD where the card read happened (some PDs have two
    /// physical card readers to put on either side of a door).
    ///
    /// false - Forward
    /// true - Backward
    pub direction: bool,

    /// Number of valid data bits in [`OsdpEventCardRead::data`] when the card
    /// format is not [`OsdpCardFormats::Ascii`]. For [`OsdpCardFormats::Ascii`], this
    /// field is set to zero.
    pub nr_bits: usize,

    /// Card data; bytes or bits depending on [`OsdpCardFormats`]
    pub data: Vec<u8>,
}

impl OsdpEventCardRead {
    /// Create an raw data card read event for self and direction set to forward
    pub fn new_raw(data: Vec<u8>) -> Self {
        Self {
            reader_no: 0,
            format: OsdpCardFormats::Unspecified,
            direction: false,
            nr_bits: data.len() * 8,
            data,
        }
    }

    /// Create a Wiegand card read event for self and direction set to forward
    pub fn new_wiegand(nr_bits: usize, data: Vec<u8>) -> Result<Self> {
        if nr_bits > data.len() * 8 {
            return Err(OsdpError::Command);
        }
        Ok(Self {
            reader_no: 0,
            format: OsdpCardFormats::Wiegand,
            direction: false,
            nr_bits,
            data,
        })
    }
}

impl TryFrom<libosdp_sys::osdp_event_cardread> for OsdpEventCardRead {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_event_cardread) -> Result<Self> {
        let format = value.format.try_into()?;
        let len = value.length as usize;
        let (nr_bits, nr_bytes) = (len, len.div_ceil(8));

        Ok(OsdpEventCardRead {
            reader_no: value.reader_no,
            format,
            direction: value.direction == 1,
            nr_bits,
            data: value.data[0..nr_bytes].to_vec(),
        })
    }
}

impl From<OsdpEventCardRead> for libosdp_sys::osdp_event_cardread {
    fn from(value: OsdpEventCardRead) -> Self {
        let mut data = [0; libosdp_sys::OSDP_EVENT_CARDREAD_MAX_DATALEN as usize];
        let length = value.nr_bits as i32;
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_event_cardread {
            reader_no: value.reader_no,
            format: value.format.into(),
            direction: value.direction as i32,
            length,
            data,
        }
    }
}

/// Event to describe a key press activity on the PD
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventKeyPress {
    /// Reader (another device connected to this PD) which caused this event
    ///
    /// 0 - self
    /// 1 - fist connected reader
    /// 2 - second connected reader
    /// ....
    pub reader_no: i32,

    /// Key data
    pub data: Vec<u8>,
}

impl OsdpEventKeyPress {
    /// Create key press event for the keys specified in `data`.
    pub fn new(data: Vec<u8>) -> Self {
        Self { reader_no: 0, data }
    }
}

impl From<libosdp_sys::osdp_event_keypress> for OsdpEventKeyPress {
    fn from(value: libosdp_sys::osdp_event_keypress) -> Self {
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        OsdpEventKeyPress {
            reader_no: value.reader_no,
            data,
        }
    }
}

impl From<OsdpEventKeyPress> for libosdp_sys::osdp_event_keypress {
    fn from(value: OsdpEventKeyPress) -> Self {
        let mut data = [0; libosdp_sys::OSDP_EVENT_KEYPRESS_MAX_DATALEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_event_keypress {
            reader_no: value.reader_no,
            length: value.data.len() as i32,
            data,
        }
    }
}

/// Event to transport a Manufacturer specific command's response.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventMfgReply {
    /// 3-byte IEEE assigned OUI used as vendor code
    pub vendor_code: (u8, u8, u8),

    /// Reply data (if any)
    pub data: Vec<u8>,
}

impl From<libosdp_sys::osdp_event_mfgrep> for OsdpEventMfgReply {
    fn from(value: libosdp_sys::osdp_event_mfgrep) -> Self {
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        let bytes = value.vendor_code.to_le_bytes();
        let vendor_code: (u8, u8, u8) = (bytes[0], bytes[1], bytes[2]);
        OsdpEventMfgReply {
            vendor_code,
            data,
        }
    }
}

impl From<OsdpEventMfgReply> for libosdp_sys::osdp_event_mfgrep {
    fn from(value: OsdpEventMfgReply) -> Self {
        let mut data = [0; libosdp_sys::OSDP_EVENT_MFGREP_MAX_DATALEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        libosdp_sys::osdp_event_mfgrep {
            vendor_code: value.vendor_code.as_le(),
            length: value.data.len() as u8,
            data,
        }
    }
}

/// Status report type
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OsdpStatusReportType {
    /// Input status report
    Input,
    /// Output status report
    Output,
    /// Remote status report
    Remote,
    /// Local status report
    Local,
}

impl TryFrom<libosdp_sys::osdp_status_report_type> for OsdpStatusReportType {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_status_report_type) -> Result<Self> {
        match value {
            libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_INPUT => {
                Ok(OsdpStatusReportType::Input)
            }
            libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_OUTPUT => {
                Ok(OsdpStatusReportType::Output)
            }
            libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_REMOTE => {
                Ok(OsdpStatusReportType::Remote)
            }
            libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_LOCAL => {
                Ok(OsdpStatusReportType::Local)
            }
            rt => Err(OsdpError::Parse(format!("Unknown report type ({rt})").into())),
        }
    }
}

impl From<OsdpStatusReportType> for libosdp_sys::osdp_status_report_type {
    fn from(value: OsdpStatusReportType) -> Self {
        match value {
            OsdpStatusReportType::Input => {
                libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_INPUT
            }
            OsdpStatusReportType::Output => {
                libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_OUTPUT
            }
            OsdpStatusReportType::Remote => {
                libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_REMOTE
            }
            OsdpStatusReportType::Local => {
                libosdp_sys::osdp_status_report_type_OSDP_STATUS_REPORT_LOCAL
            }
        }
    }
}

/// Event to describe various status changes on PD
///
/// This event is used by the PD to indicate status such as input, output,
/// tamper, etc.,. up to a maximum of 64 status bits can be reported. The values
/// of the least significant N bit of status are considered, where N is the
/// number of items as described in the corresponding capability codes,
/// - PdCapability::OutputControl
/// - PdCapability::ContactStatusMonitoring
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpStatusReport {
    /// The kind of event to report see `enum osdp_event_status_type_e`
    pub type_: OsdpStatusReportType,
    /// Number of valid entries in `report`
    pub nr_entries: usize,
    /// Status report
    #[serde(with = "serde_arrays")]
    pub report: [u8; 64],
}

impl TryFrom<libosdp_sys::osdp_status_report> for OsdpStatusReport {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_status_report) -> Result<Self> {
        Ok(OsdpStatusReport {
            type_: value.type_.try_into()?,
            nr_entries: value.nr_entries as usize,
            report: value.report,
        })
    }
}

impl From<OsdpStatusReport> for libosdp_sys::osdp_status_report {
    fn from(value: OsdpStatusReport) -> Self {
        libosdp_sys::osdp_status_report {
            type_: value.type_.into(),
            nr_entries: value.nr_entries as i32,
            report: value.report,
        }
    }
}

/// Extended write reply - Error
///
/// This may be sent as a poll response, or in response to any Mode -00
/// command (osdp_XWR|XRD_MODE=0|XWR_PCMND=any) to return an error or
/// negative acknowledge (NAK) condition.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventErrorData {
    /// Error code
    pub error_code: u8,
}

impl From<libosdp_sys::osdp_xrd_error_reply> for OsdpEventErrorData {
    fn from(value: libosdp_sys::osdp_xrd_error_reply) -> Self {
        Self {
            error_code: value.error_code,
        }
    }
}

impl From<OsdpEventErrorData> for libosdp_sys::osdp_xrd_error_reply {
    fn from(value: OsdpEventErrorData) -> Self {
        Self {
            error_code: value.error_code,
        }
    }
}

/// Extended write reply - Mode setting report
///
/// This is sent in response to osdp_XWR|XRD_MODE=0|XWR_PCMND=2
/// and it returns its current background behavior mode setting
/// and configuration in response to the request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventModeReportData {
    /// Extended write background operation mode in effect
    pub mode_code: u8,
    /// Mode configuration data
    pub mode_config: u8,
}

impl From<libosdp_sys::osdp_xrd_mode_report> for OsdpEventModeReportData {
    fn from(value: libosdp_sys::osdp_xrd_mode_report) -> Self {
        Self {
            mode_code: value.mode_code,
            mode_config: value.mode_config,
        }
    }
}

impl From<OsdpEventModeReportData> for libosdp_sys::osdp_xrd_mode_report {
    fn from(value: OsdpEventModeReportData) -> Self {
        Self {
            mode_code: value.mode_code,
            mode_config: value.mode_config,
        }
    }
}

/// Extended write reply - Card information report
///
/// When enabled, this reply is sent in response to an osdp_POLL command
/// after a smart card is detected that may require additional processing
/// in an alternate mode.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventCardReportData {
    /// Reader number
    pub reader: u8,
    /// Card protocol:
    ///  - 0x00 - Contact T0/T1
    ///  - 0x01 - ISO 14443 A/B
    ///  - 0x02 - Reserved for future use
    pub protocol: u8,
    /// Card serial number
    pub csn: Vec<u8>,
    /// Protocol data:
    ///  - Protocol 0: ATR
    ///  - Protocol 1: ATS/ATQB
    pub data: Vec<u8>,
}

impl From<libosdp_sys::osdp_xrd_card_report> for OsdpEventCardReportData {
    fn from(value: libosdp_sys::osdp_xrd_card_report) -> Self {
        let n = value.csn_length as usize;
        let csn = value.csn[0..n].to_vec();
        let n = value.length as usize;
        let data = value.data[0..n].to_vec();
        Self {
            reader: value.reader,
            protocol: value.protocol,
            csn,
            data,
        }
    }
}

impl From<OsdpEventCardReportData> for libosdp_sys::osdp_xrd_card_report {
    fn from(value: OsdpEventCardReportData) -> Self {
        let mut csn = [0; libosdp_sys::OSDP_EVENT_XRD_CSN_MAX_DATALEN as usize];
        csn[..value.csn.len()].copy_from_slice(&value.csn[..]);
        let mut data = [0; libosdp_sys::OSDP_EVENT_XRD_PROTOCOL_MAX_LEN as usize];
        data[..value.data.len()].copy_from_slice(&value.data[..]);
        Self {
            reader: value.reader,
            protocol: value.protocol,
            csn_length: value.csn.len() as u8,
            csn,
            length: value.data.len() as u8,
            data,
        }
    }
}

/// Extended write reply - Card present notification
///
/// This reply is sent in response to n osdp_PR01SCSCAN indicating the
/// resulting smart card connection status or sent in response to an osdp_POLL.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventCardPresentData {
    /// Reader number
    pub reader: u8,
    /// Smart Card Present Status:
    ///  - 0x00 - Card not present.
    ///  - 0x01 - Card present but interface not specified.
    ///  - 0x02 - Card present on contactless interface.
    ///  - 0x03 - Card present on contact interface.
    ///  - 0x04 - Reserved for future use.
    pub status: u8,
}

impl From<libosdp_sys::osdp_xrd_card_present> for OsdpEventCardPresentData {
    fn from(value: libosdp_sys::osdp_xrd_card_present) -> Self {
        Self {
            reader: value.reader,
            status: value.status,
        }
    }
}

impl From<OsdpEventCardPresentData> for libosdp_sys::osdp_xrd_card_present {
    fn from(value: OsdpEventCardPresentData) -> Self {
        Self {
            reader: value.reader,
            status: value.status,
        }
    }
}

/// Extended write reply - Transparent card data
///
/// This reply is sent in response to a XWR_PCMND Code 0x01 “XMIT” reporting
/// a data packet received from a smart card by a reader set to operate in
/// background Mode = 1.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventTransparentCardData {
    /// Reader number
    pub reader: u8,
    /// Results of requested command
    pub status: u8,
    /// APDU data from the smart card
    pub apdu: Vec<u8>,
}

impl From<libosdp_sys::osdp_xrd_card_data> for OsdpEventTransparentCardData {
    fn from(value: libosdp_sys::osdp_xrd_card_data) -> Self {
        let n = value.apdu_length as usize;
        let apdu = value.apdu[0..n].to_vec();
        Self {
            reader: value.reader,
            status: value.status,
            apdu,
        }
    }
}

impl From<OsdpEventTransparentCardData> for libosdp_sys::osdp_xrd_card_data {
    fn from(value: OsdpEventTransparentCardData) -> Self {
        let mut apdu = [0; libosdp_sys::OSDP_EVENT_XRD_APDU_MAX_DATALEN as usize];
        apdu[..value.apdu.len()].copy_from_slice(&value.apdu[..]);
        Self {
            reader: value.reader,
            status: value.status,
            apdu_length: value.apdu.len() as u8,
            apdu,
        }
    }
}

/// Extended write reply - Secure PIN entry complete
///
/// This reply is sent in response to an XWR_PCMND Code 0x03 “Secure PIN Entry”
/// indicating that a Secure Pin Entry (SPE) sequence has completed.
/// This reply is used by smart card readers set to operate in background
/// Mode = 1.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OsdpEventPinCompleteData {
    /// Reader number
    pub reader: u8,
    /// Results of the SPE sequence
    pub status: u8,
    /// Number of attempts before card "locks" itself
    pub tries: u8,
}

impl From<libosdp_sys::osdp_xrd_pin_complete> for OsdpEventPinCompleteData {
    fn from(value: libosdp_sys::osdp_xrd_pin_complete) -> Self {
        Self {
            reader: value.reader,
            status: value.status,
            tries: value.tries,
        }
    }
}

impl From<OsdpEventPinCompleteData> for libosdp_sys::osdp_xrd_pin_complete {
    fn from(value: OsdpEventPinCompleteData) -> Self {
        Self {
            reader: value.reader,
            status: value.status,
            tries: value.tries,
        }
    }
}


/// Extended read event types
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OsdpEventXReadType {
    /// General error indication: PD was unable to process the command.
    Error,
    /// The current extended write mode in effect.
    ModeReport,
    /// A card information report of a detected smart card.
    CardReport,
    /// Card present notification.
    CardPresent,
    /// Transparent card data.
    CardData,
    /// Secure PIN entry complete.
    PinComplete,
}
impl OsdpEventXReadType {
    /// Convert extended write mode and reply code to an enumerated type.
    pub fn try_from_mode_and_reply(mode: u8, reply: u8) -> Option<Self> {
        match (mode, reply) {
            (_, 0) => Some(Self::Error),
            (0, 1) => Some(Self::ModeReport),
            (0, 2) => Some(Self::CardReport),
            (1, 1) => Some(Self::CardPresent),
            (1, 2) => Some(Self::CardData),
            (1, 3) => Some(Self::PinComplete),
            (_, _) => None,
        }
    }

    /// Get extended write mode and reply code from enumerated type.
    pub fn to_mode_and_reply(self) -> (u8, u8) {
        match self {
            Self::Error => (0, 0),
            Self::ModeReport => (0, 1),
            Self::CardReport => (0, 2),
            Self::CardPresent => (1, 1),
            Self::CardData => (1, 2),
            Self::PinComplete => (1, 3),
        }
    }
}

impl From<&OsdpEventXRead> for OsdpEventXReadType {
    fn from(value: &OsdpEventXRead) -> Self {
        match value {
            OsdpEventXRead::Error(_) => Self::Error,
            OsdpEventXRead::ModeReport(_) => Self::ModeReport,
            OsdpEventXRead::CardReport(_) => Self::CardReport,
            OsdpEventXRead::CardPresent(_) => Self::CardPresent,
            OsdpEventXRead::CardData(_) => Self::CardData,
            OsdpEventXRead::PinComplete(_) => Self::PinComplete,
        }
    }
}

impl From<OsdpEventXRead> for libosdp_sys::osdp_event_xread {
    fn from(value: OsdpEventXRead) -> Self {
        let (mode, reply) = OsdpEventXReadType::from(&value).to_mode_and_reply();
        match value {
            OsdpEventXRead::Error(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        error_reply: c.clone().into(),
                    },
                }
            },
            OsdpEventXRead::ModeReport(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        mode_report: c.clone().into(),
                    },
                }
            },
            OsdpEventXRead::CardReport(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        card_report: c.clone().into(),
                    },
                }
            },
            OsdpEventXRead::CardPresent(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        card_present: c.clone().into(),
                    },
                }
            },
            OsdpEventXRead::CardData(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        card_data: c.clone().into(),
                    },
                }
            },
            OsdpEventXRead::PinComplete(c) => {
                Self {
                    mode,
                    reply,
                    __bindgen_anon_1: libosdp_sys::osdp_event_xread__bindgen_ty_1 {
                        pin_complete: c.clone().into(),
                    },
                }
            },
        }
    }
}

impl TryFrom<libosdp_sys::osdp_event_xread> for OsdpEventXRead {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_event_xread) -> Result<Self> {
        OsdpEventXReadType::try_from_mode_and_reply(value.mode, value.reply)
            .map(|t| {
                match t {
                    OsdpEventXReadType::Error => {
                        Self::Error(unsafe { value.__bindgen_anon_1.error_reply.into() })
                    },
                    OsdpEventXReadType::ModeReport => {
                        Self::ModeReport(unsafe { value.__bindgen_anon_1.mode_report.into() })
                    },
                    OsdpEventXReadType::CardReport => {
                        Self::CardReport(unsafe { value.__bindgen_anon_1.card_report.into() })
                    },
                    OsdpEventXReadType::CardPresent => {
                        Self::CardPresent(unsafe { value.__bindgen_anon_1.card_present.into() })
                    },
                    OsdpEventXReadType::CardData => {
                        Self::CardData(unsafe { value.__bindgen_anon_1.card_data.into() })
                    },
                    OsdpEventXReadType::PinComplete => {
                        Self::PinComplete(unsafe { value.__bindgen_anon_1.pin_complete.into() })
                    },
                }
            })
            .ok_or(OsdpError::Parse(
                "Unknown extended write mode and reply combination".into(),
            ))
    }
}


/// Event to describe an extended write command response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OsdpEventXRead {
    /// General error indication: PD was unable to process the command.
    Error(OsdpEventErrorData),
    /// The current extended write mode in effect.
    ModeReport(OsdpEventModeReportData),
    /// A card information report of a detected smart card.
    CardReport(OsdpEventCardReportData),
    /// Card present notification.
    CardPresent(OsdpEventCardPresentData),
    /// Transparent card data.
    CardData(OsdpEventTransparentCardData),
    /// Secure PIN entry complete.
    PinComplete(OsdpEventPinCompleteData),
}


/// CP to intimate it about various events that originate there (such as key
/// press, card reads, etc.,). They do this by creating an “event” and sending
/// it to the CP. This module is responsible to handling such events though
/// OsdpEvent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OsdpEvent {
    /// Event that describes card read activity on the PD
    CardRead(OsdpEventCardRead),

    /// Event to describe a key press activity on the PD
    KeyPress(OsdpEventKeyPress),

    /// Event to transport a Manufacturer specific command’s response
    MfgReply(OsdpEventMfgReply),

    /// Event to describe a input/output/tamper/power status change
    Status(OsdpStatusReport),

    /// Event to describe an extended write command response
    ExtendedRead(OsdpEventXRead),
}

impl From<OsdpEvent> for libosdp_sys::osdp_event {
    fn from(value: OsdpEvent) -> Self {
        match value {
            OsdpEvent::CardRead(e) => libosdp_sys::osdp_event {
                _node: unsafe { core::mem::zeroed() },
                type_: libosdp_sys::osdp_event_type_OSDP_EVENT_CARDREAD,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_event__bindgen_ty_1 {
                    cardread: e.clone().into(),
                },
            },
            OsdpEvent::KeyPress(e) => libosdp_sys::osdp_event {
                _node: unsafe { core::mem::zeroed() },
                type_: libosdp_sys::osdp_event_type_OSDP_EVENT_KEYPRESS,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_event__bindgen_ty_1 {
                    keypress: e.clone().into(),
                },
            },
            OsdpEvent::MfgReply(e) => libosdp_sys::osdp_event {
                _node: unsafe { core::mem::zeroed() },
                type_: libosdp_sys::osdp_event_type_OSDP_EVENT_MFGREP,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_event__bindgen_ty_1 {
                    mfgrep: e.clone().into(),
                },
            },
            OsdpEvent::Status(e) => libosdp_sys::osdp_event {
                _node: unsafe { core::mem::zeroed() },
                type_: libosdp_sys::osdp_event_type_OSDP_EVENT_STATUS,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_event__bindgen_ty_1 { status: e.into() },
            },
            OsdpEvent::ExtendedRead(e) => libosdp_sys::osdp_event {
                _node: unsafe { core::mem::zeroed() },
                type_: libosdp_sys::osdp_event_type_OSDP_EVENT_XREAD,
                flags: 0,
                __bindgen_anon_1: libosdp_sys::osdp_event__bindgen_ty_1 { xread: e.into() },
            }
        }
    }
}

impl TryFrom<libosdp_sys::osdp_event> for OsdpEvent {
    type Error = OsdpError;

    fn try_from(value: libosdp_sys::osdp_event) -> Result<Self> {
        match value.type_ {
            libosdp_sys::osdp_event_type_OSDP_EVENT_CARDREAD => {
                let data = unsafe { value.__bindgen_anon_1.cardread.try_into() }?;
                Ok(OsdpEvent::CardRead(data))
            }
            libosdp_sys::osdp_event_type_OSDP_EVENT_KEYPRESS => {
                Ok(OsdpEvent::KeyPress(unsafe { value.__bindgen_anon_1.keypress.into() }))
            }
            libosdp_sys::osdp_event_type_OSDP_EVENT_MFGREP => {
                Ok(OsdpEvent::MfgReply(unsafe { value.__bindgen_anon_1.mfgrep.into() }))
            }
            libosdp_sys::osdp_event_type_OSDP_EVENT_STATUS => {
                let data = unsafe { value.__bindgen_anon_1.status.try_into() }?;
                Ok(OsdpEvent::Status(data))
            }
            et => Err(OsdpError::Parse(format!("Unknown event ({et})").into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OsdpEventCardRead;
    use libosdp_sys::{
        osdp_event_cardread, osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_UNSPECIFIED,
        osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_WIEGAND,
    };

    #[test]
    fn test_event_cardread() {
        let event = OsdpEventCardRead::new_raw(vec![0x55, 0xAA]);
        let event_struct: osdp_event_cardread = event.clone().into();

        assert_eq!(event_struct.length, 2 * 8);
        assert_eq!(event_struct.direction, 0);
        assert_eq!(
            event_struct.format,
            osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_UNSPECIFIED
        );
        assert_eq!(event_struct.data[0], 0x55);
        assert_eq!(event_struct.data[1], 0xAA);

        assert_eq!(event, event_struct.try_into().unwrap());

        let event = OsdpEventCardRead::new_wiegand(15, vec![0x55, 0xAA]).unwrap();
        let event_struct: osdp_event_cardread = event.clone().into();

        assert_eq!(event_struct.length, 15);
        assert_eq!(event_struct.direction, 0);
        assert_eq!(
            event_struct.format,
            osdp_event_cardread_format_e_OSDP_CARD_FMT_RAW_WIEGAND
        );
        assert_eq!(event_struct.data[0], 0x55);
        assert_eq!(event_struct.data[1], 0xAA);

        assert_eq!(event, event_struct.try_into().unwrap());
    }
}
