typedef enum logic [2:0] {
    STATE_IDLE  = 3'd0,
    STATE_BUSY  = 3'd2,
    STATE_DONE  = 3'd5,
    STATE_ERROR = 3'd7
} state_t;

typedef enum logic signed [3:0] {
    SIGNED_NEG  = 4'hD,
    SIGNED_ZERO = 4'h0,
    SIGNED_POS  = 4'h5
} signed_state_t;

module packed_enum_ports (
    input logic clk,

    input  state_t state,
    output state_t state_out,
    output logic [2:0] state_raw_out,

    input  signed_state_t signed_state,
    output signed_state_t signed_state_out,
    output logic signed [3:0] signed_state_raw_out
);

always_ff @(posedge clk) begin
    state_out <= state;
    state_raw_out <= state;

    signed_state_out <= signed_state;
    signed_state_raw_out <= signed_state;
end

endmodule
