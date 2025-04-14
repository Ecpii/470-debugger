/**
 * HEADERS variables that specify the columns to display for different
 * structs and how to fetch/format them.
 */
use crate::utils::{parse_mem_size, Column, DisplayType};

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
typedef struct packed {
  logic valid;
  logic taken;          // was the branch taken?
  ADDR pc;              // pc of the branch instruction
  ADDR target_pc;       // pc we should go to after this branch instruction
  reg_idx_t rd;
  bmask_t bid;
  rob_num_t rob_num;
  branch_pred_packet branch_packet;
} branch_output_t;
 */
pub const BRANCH_OUTPUT_HEADERS: [Column; 7] = [
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
        name: "taken",
        key: Some("taken"),
        width: 5,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "pc",
        key: Some("pc"),
        width: 6,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "target_pc",
        key: Some("pc"),
        width: 9,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "bid",
        key: Some("bid"),
        width: 7,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "pht_index",
        key: Some("branch_packet.pht_index"),
        width: 9,
        display_type: DisplayType::Decimal,
    },
];

/*
typedef struct packed {
  logic valid; // whether or not we actually are waiting for something
  logic ready; // complete_data holds correct data that can be sent to the output

  MEM_TAG mem_tag; // tag we are waiting for from memory

  bmask_t bmask;

  // query data
  DCACHE_ADDR addr; // original addr, not block aligned
                    // the block we request and get back will be block aligned i think though
                    // stored just in case we do something fancy later
  MEM_SIZE size;
  logic is_store;
  DATA store_data;

  DATA complete_data;
} dcache_mshr_t;
 */
pub const MSHR_HEADERS: [Column; 7] = [
    Column {
        name: "#",
        key: None,
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "mem_tag",
        key: Some("mem_tag"),
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
        name: "addr",
        key: None,
        width: 5,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "size",
        key: Some("size"),
        width: 6,
        display_type: DisplayType::Custom(parse_mem_size),
    },
    Column {
        name: "is_store",
        key: Some("is_store"),
        width: 8,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "store_data",
        key: Some("store_data"),
        width: 10,
        display_type: DisplayType::Hex,
    },
];

pub const DCACHE_META_HEADERS: [Column; 8] = [
    Column {
        name: "#",
        key: None,
        width: 2,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "set_num",
        key: None,
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "set_idx",
        key: None,
        width: 7,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "dirty",
        key: Some("dirty"),
        width: 5,
        display_type: DisplayType::Binary,
    },
    Column {
        name: "tag",
        key: Some("tag"),
        width: 6,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "addr",
        key: None,
        width: 5,
        display_type: DisplayType::Hex,
    },
    Column {
        name: "lru",
        key: Some("lru"),
        width: 3,
        display_type: DisplayType::Decimal,
    },
    Column {
        name: "data",
        key: None,
        width: 18,
        display_type: DisplayType::Hex,
    },
];
