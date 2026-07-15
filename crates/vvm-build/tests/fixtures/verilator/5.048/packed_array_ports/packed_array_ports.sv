module packed_array_ports (
    input  logic             clk,
    input  logic [3:0][7:0] packed_bytes,

    output logic [3:0][7:0] packed_bytes_out,
    output logic [7:0]      first_byte,
    output logic [7:0]      last_byte
);

always_ff @(posedge clk) begin
    packed_bytes_out <= packed_bytes;
    first_byte       <= packed_bytes[0];
    last_byte        <= packed_bytes[3];
end

endmodule
