module signed_ports (
    input  logic signed        input_i1,
    input  logic signed [4:0]  input_i5,
    input  logic signed [8:0]  input_i9,
    input  logic signed [16:0] input_i17,
    input  logic signed [32:0] input_i33,
    input  logic signed [63:0] input_i64,

    output logic signed        output_i1,
    output logic signed [4:0]  output_i5,
    output logic signed [8:0]  output_i9,
    output logic signed [16:0] output_i17,
    output logic signed [32:0] output_i33,
    output logic signed [63:0] output_i64
);

always_comb begin
    output_i1  = input_i1;
    output_i5  = input_i5;
    output_i9  = input_i9;
    output_i17 = input_i17;
    output_i33 = input_i33;
    output_i64 = input_i64;
end

endmodule
