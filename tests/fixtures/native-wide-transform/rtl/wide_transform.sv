module wide_transform (
    input  logic                 clk,
    input  logic         [128:0] value,
    input  logic signed  [128:0] signed_value,

    output logic         [128:0] shifted,
    output logic         [31:0]  low_word,
    output logic                 high_bit,
    output logic signed  [128:0] signed_mirror,
    output logic                 signed_sign
);

always_ff @(posedge clk) begin
    shifted      <= {value[127:0], 1'b0};
    low_word     <= value[31:0];
    high_bit     <= value[128];

    signed_mirror <= signed_value;
    signed_sign   <= signed_value[128];
end

endmodule
