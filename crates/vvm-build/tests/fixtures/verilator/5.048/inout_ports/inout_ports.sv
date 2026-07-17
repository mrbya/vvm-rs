module inout_ports (
    input logic pin_drive_enable,
    input logic pin_drive_value,
    input logic [7:0] bus_drive_enable,
    input logic [7:0] bus_drive_value,
    input logic signed_drive_enable,
    input logic signed [15:0] signed_drive_value,
    input logic wide_drive_enable,
    input logic [127:0] wide_drive_value,
    inout wire pin,
    inout wire [7:0] bus,
    inout wire signed [15:0] signed_bus,
    inout wire [127:0] wide_bus
);

genvar bit_index;

generate
    for (bit_index = 0; bit_index < 8; bit_index++) begin : generate_bus_driver
        assign bus[bit_index] =
            bus_drive_enable[bit_index] ? bus_drive_value[bit_index] : 1'bz;
    end
endgenerate

assign pin = pin_drive_enable ? pin_drive_value : 1'bz;
assign signed_bus = signed_drive_enable ? signed_drive_value : 'z;
assign wide_bus = wide_drive_enable ? wide_drive_value : 'z;

endmodule
