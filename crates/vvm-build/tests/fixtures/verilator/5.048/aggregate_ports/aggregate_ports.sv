typedef struct packed {
    logic [3:0] opcode;
    logic       valid;
    logic [2:0] flags;
    logic [7:0] payload;
} packet_t;

typedef enum logic [1:0] {
    STATE_IDLE = 2'd0,
    STATE_BUSY = 2'd1,
    STATE_DONE = 2'd2,
    STATE_ERR  = 2'd3
} state_t;

module aggregate_ports (
    input  logic              clk,

    input  logic [3:0][7:0]  packed_bytes,
    output logic [3:0][7:0]  packed_bytes_out,

    input  packet_t           packet,
    output packet_t           packet_out,

    input  state_t            state,
    output state_t            state_out,

    input  logic [7:0]        unpacked_bytes [4],
    output logic [7:0]        unpacked_bytes_out [4]
);

always_ff @(posedge clk) begin
    packed_bytes_out <= packed_bytes;
    packet_out       <= packet;
    state_out        <= state;

    for (int i = 0; i < 4; i++) begin
        unpacked_bytes_out[i] <= unpacked_bytes[i];
    end
end

endmodule
