use crate::utils::{Column, DisplayType};

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
        name: "bmask",
        key: Some("bmask"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "rob_num",
        key: Some("rob_num"),
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "sq_tag",
        key: Some("store_queue_tag"),
        width: 7,
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
