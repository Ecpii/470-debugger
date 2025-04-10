/**
 * HEADERS variables that specify the columns to display for different
 * structs and how to fetch/format them.
 */
use crate::utils::{Column, DisplayType};

/*
typedef struct packed {
  logic valid;
  DATA rs1_val;
  DATA rs2_val;
  reg_idx_t rd;
  op_info_t op;
  bmask_t bmask;
  rob_num_t rob_num;
  store_queue_num_t store_queue_tag;
  logic [3:0] mem_blocks;
} fu_input_packet_t;
*/
pub const FU_INPUT_HEADERS: [Column; 8] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 3,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 6,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_blocks",
        key: Some("mem_blocks"),
        width: 10,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "rs1_val",
        key: Some("rs1_val"),
        width: 10,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "rs2_val",
        key: Some("rs2_val"),
        width: 10,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "op",
        key: None,
        width: 20,
        display_type: DisplayType::Binary,
    },
];

/*
typedef struct packed {
  logic valid;
  op_info_t op;
  bmask_t bmask;
  reg_idx_t rd;
  rob_num_t rob_num;
  store_queue_num_t store_queue_tag;
  logic [3:0] mem_blocks;
  DATA alu_result;
  DATA mem_data;
} fu_output_packet_t;
*/
pub const FU_OUTPUT_HEADERS: [Column; 8] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 6,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_blocks",
        key: Some("mem_blocks"),
        width: 10,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "alu_result",
        key: Some("alu_result"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "mem_data",
        key: Some("mem_data"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "op",
        key: None,
        width: 20,
        display_type: DisplayType::Binary,
    },
];

/*
 DATA data;
 ADDR addr;
 op_info_t op;
 rob_num_t rob_num;
 bmask_t bmask;
 reg_idx_t rd;
 logic wr;
 logic valid;
 logic [3:0] mem_blocks;
 store_queue_num_t store_queue_tag;
*/
pub const MEM_INPUT_HEADERS: [Column; 9] = [
    Column {
        name: "rd",
        key: Some("rd"),
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 6,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_blocks",
        key: Some("mem_blocks"),
        width: 10,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "wr",
        key: Some("wr"),
        width: 2,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "addr",
        key: Some("addr"),
        width: 8,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "data",
        key: Some("data"),
        width: 16,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "op",
        key: None,
        width: 20,
        display_type: DisplayType::Binary,
    },
];
