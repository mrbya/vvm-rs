// Split inout model: Rust resolves data before it is sampled by the DUT.
module tri_state_bus (
    input logic [7:0] drive_enable,
    input logic [7:0] drive_value,
    inout wire [7:0] data,
    output wire [7:0] sampled_data
);

genvar bit_index;

generate
    for (bit_index = 0; bit_index < 8; bit_index++) begin : generate_data_driver
        assign data[bit_index] = drive_enable[bit_index] ? drive_value[bit_index] : 1'bz;
    end
endgenerate

assign sampled_data = data;

endmodule
