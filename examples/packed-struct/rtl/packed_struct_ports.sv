typedef struct packed {
    logic        [3:0]  opcode;
    logic               valid;
    logic signed [6:0]  delta;
    logic        [3:0]  flags;
    logic        [15:0] payload;
} packet_t;

typedef struct packed {
    logic        [6:0]   tag;
    logic signed [128:0] payload;
} wide_packet_t;

module packed_struct_ports (
    input  logic clk,

    input  packet_t packet,
    output packet_t packet_out,

    output logic        [3:0]  opcode_out,
    output logic               valid_out,
    output logic signed [6:0]  delta_out,
    output logic        [3:0]  flags_out,
    output logic        [15:0] payload_out,

    input  wide_packet_t wide_packet,
    output wide_packet_t wide_packet_out,

    output logic        [6:0]   wide_tag_out,
    output logic signed [128:0] wide_payload_out
);

always_ff @(posedge clk) begin
    packet_out <= packet;
    opcode_out <= packet.opcode;
    valid_out <= packet.valid;
    delta_out <= packet.delta;
    flags_out <= packet.flags;
    payload_out <= packet.payload;

    wide_packet_out <= wide_packet;
    wide_tag_out <= wide_packet.tag;
    wide_payload_out <= wide_packet.payload;
end

endmodule
