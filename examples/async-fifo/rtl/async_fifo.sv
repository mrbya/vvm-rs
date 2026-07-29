// Conventional dual-clock FIFO with pointer synchronization, not a CDC proof.
module async_fifo (
    input  logic       wr_clk,
    input  logic       rd_clk,
    input  logic       wr_reset_n,
    input  logic       rd_reset_n,
    input  logic       write,
    input  logic [7:0] write_data,
    input  logic       read,
    output logic [7:0] read_data,
    output logic       full,
    output logic       empty
);
    // Shared storage, local pointers, and synchronized remote Gray pointers.
    logic [7:0] memory [0:7];
    logic [3:0] write_binary;
    logic [3:0] write_gray;
    logic [3:0] read_binary;
    logic [3:0] read_gray;
    logic [3:0] read_gray_sync_1;
    logic [3:0] read_gray_sync_2;
    logic [3:0] write_gray_sync_1;
    logic [3:0] write_gray_sync_2;

    logic [3:0] write_binary_next;
    logic [3:0] write_gray_next;
    logic [3:0] read_binary_next;
    logic [3:0] read_gray_next;

    // Binary pointers index memory locally; Gray pointers cross clock domains.
    assign write_binary_next = write_binary + (write && !full);
    assign write_gray_next = (write_binary_next >> 1) ^ write_binary_next;
    assign read_binary_next = read_binary + (read && !empty);
    assign read_gray_next = (read_binary_next >> 1) ^ read_binary_next;

    always_ff @(posedge wr_clk or negedge wr_reset_n) begin
        if (!wr_reset_n) begin
            write_binary <= 0;
            write_gray <= 0;
            read_gray_sync_1 <= 0;
            read_gray_sync_2 <= 0;
            full <= 0;
        end else begin
            // Two flip-flops reduce the direct asynchronous-pointer exposure.
            read_gray_sync_1 <= read_gray;
            read_gray_sync_2 <= read_gray_sync_1;
            if (write && !full) begin
                memory[write_binary[2:0]] <= write_data;
            end
            write_binary <= write_binary_next;
            write_gray <= write_gray_next;
            // A Gray pointer is full when its two high bits invert at depth.
            full <= write_gray_next == {~read_gray_sync_2[3:2], read_gray_sync_2[1:0]};
        end
    end

    always_ff @(posedge rd_clk or negedge rd_reset_n) begin
        if (!rd_reset_n) begin
            read_binary <= 0;
            read_gray <= 0;
            write_gray_sync_1 <= 0;
            write_gray_sync_2 <= 0;
            read_data <= 0;
            empty <= 1;
        end else begin
            // Read-domain copy of the write pointer uses the same two-stage crossing.
            write_gray_sync_1 <= write_gray;
            write_gray_sync_2 <= write_gray_sync_1;
            if (read && !empty) begin
                read_data <= memory[read_binary[2:0]];
            end
            read_binary <= read_binary_next;
            read_gray <= read_gray_next;
            empty <= read_gray_next == write_gray_sync_2;
        end
    end
endmodule
