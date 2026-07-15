module unpacked_array_ports (
    input logic clk,
    input logic flags [0:3],
    output logic flags_out [0:3],
    output logic first_flag_out,
    output logic last_flag_out,
    input logic [7:0] bytes [3:0],
    output logic [7:0] bytes_out [3:0],
    output logic [7:0] bytes_left_out,
    output logic [7:0] bytes_right_out,
    input logic signed [12:0] signed_values [0:2],
    output logic signed [12:0] signed_values_out [0:2],
    input logic [128:0] wide_values [1:0],
    output logic [128:0] wide_values_out [1:0],
    output logic [31:0] wide_left_low_word_out,
    output logic wide_right_high_bit_out
);
integer i;
always_ff @(posedge clk) begin
    for (i = 0; i < 4; i++) begin
        flags_out[i] <= flags[i];
        bytes_out[i] <= bytes[i];
    end
    for (i = 0; i < 3; i++) begin
        signed_values_out[i] <= signed_values[i];
    end
    for (i = 0; i < 2; i++) begin
        wide_values_out[i] <= wide_values[i];
    end
    first_flag_out <= flags[0];
    last_flag_out <= flags[3];
    bytes_left_out <= bytes[3];
    bytes_right_out <= bytes[0];
    wide_left_low_word_out <= wide_values[1][31:0];
    wide_right_high_bit_out <= wide_values[0][128];
end
endmodule
