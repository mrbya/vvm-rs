module wide_ports (
    input  logic          [64:0]  input_u65,
    input  logic          [95:0]  input_u96,
    input  logic          [128:0] input_u129,
    input  logic          [255:0] input_u256,

    input  logic signed   [64:0]  input_i65,
    input  logic signed   [128:0] input_i129,

    output logic          [64:0]  output_u65,
    output logic          [95:0]  output_u96,
    output logic          [128:0] output_u129,
    output logic          [255:0] output_u256,

    output logic signed   [64:0]  output_i65,
    output logic signed   [128:0] output_i129
);

always_comb begin
    output_u65  = input_u65;
    output_u96  = input_u96;
    output_u129 = input_u129;
    output_u256 = input_u256;

    output_i65  = input_i65;
    output_i129 = input_i129;
end

endmodule
