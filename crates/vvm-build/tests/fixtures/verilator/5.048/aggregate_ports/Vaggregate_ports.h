// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Primary model header
//
// This header should be included by all source files instantiating the design.
// The class here is then constructed to instantiate the design.
// See the Verilator manual for examples.

#ifndef VERILATED_VAGGREGATE_PORTS_H_
#define VERILATED_VAGGREGATE_PORTS_H_  // guard

#include "verilated.h"

class Vaggregate_ports__Syms;
class Vaggregate_ports___024root;

// This class is the main interface to the Verilated model
class alignas(VL_CACHE_LINE_BYTES) Vaggregate_ports VL_NOT_FINAL : public VerilatedModel {
  private:
    // Symbol table holding complete model state (owned by this class)
    Vaggregate_ports__Syms* const vlSymsp;

  public:

    // CONSTEXPR CAPABILITIES
    // Verilated with --trace?
    static constexpr bool traceCapable = false;

    // PORTS
    // The application code writes and reads these signals to
    // propagate new values into/out from the Verilated model.
    VL_IN8(&__Vm_sig_clk,0,0);
    VL_IN(&__Vm_sig_packed_bytes,31,0);
    VL_OUT(&__Vm_sig_packed_bytes_out,31,0);
    VL_IN8(&__Vm_sig_state,1,0);
    VL_OUT8(&__Vm_sig_state_out,1,0);
    VL_IN16(&__Vm_sig_packet,15,0);
    VL_OUT16(&__Vm_sig_packet_out,15,0);
    VlUnpacked<CData/*7:0*/, 4> &__Vm_sig_unpacked_bytes;
    VlUnpacked<CData/*7:0*/, 4> &__Vm_sig_unpacked_bytes_out;

    // ACCESSORS
    // The application code should use these methods to
    // propagate new values into/out from the Verilated model
    // instead of using signal variables directly.
    decltype(__Vm_sig_clk) clk() {return __Vm_sig_clk;}
    void clk(decltype(__Vm_sig_clk) v) {__Vm_sig_clk=v;}
    decltype(__Vm_sig_packed_bytes) packed_bytes() {return __Vm_sig_packed_bytes;}
    void packed_bytes(decltype(__Vm_sig_packed_bytes) v) {__Vm_sig_packed_bytes=v;}
    decltype(__Vm_sig_packed_bytes_out) packed_bytes_out() {return __Vm_sig_packed_bytes_out;}
    void packed_bytes_out(decltype(__Vm_sig_packed_bytes_out) v) {__Vm_sig_packed_bytes_out=v;}
    decltype(__Vm_sig_state) state() {return __Vm_sig_state;}
    void state(decltype(__Vm_sig_state) v) {__Vm_sig_state=v;}
    decltype(__Vm_sig_state_out) state_out() {return __Vm_sig_state_out;}
    void state_out(decltype(__Vm_sig_state_out) v) {__Vm_sig_state_out=v;}
    decltype(__Vm_sig_packet) packet() {return __Vm_sig_packet;}
    void packet(decltype(__Vm_sig_packet) v) {__Vm_sig_packet=v;}
    decltype(__Vm_sig_packet_out) packet_out() {return __Vm_sig_packet_out;}
    void packet_out(decltype(__Vm_sig_packet_out) v) {__Vm_sig_packet_out=v;}
    decltype(__Vm_sig_unpacked_bytes) unpacked_bytes() {return __Vm_sig_unpacked_bytes;}
    void unpacked_bytes(decltype(__Vm_sig_unpacked_bytes) v) {__Vm_sig_unpacked_bytes=v;}
    decltype(__Vm_sig_unpacked_bytes_out) unpacked_bytes_out() {return __Vm_sig_unpacked_bytes_out;}
    void unpacked_bytes_out(decltype(__Vm_sig_unpacked_bytes_out) v) {__Vm_sig_unpacked_bytes_out=v;}

    // CELLS
    // Public to allow access to /* verilator public */ items.
    // Otherwise the application code can consider these internals.

    // Root instance pointer to allow access to model internals,
    // including inlined /* verilator public_flat_* */ items.
    Vaggregate_ports___024root* const rootp;

    // CONSTRUCTORS
    /// Construct the model; called by application code
    /// If contextp is null, then the model will use the default global context
    /// If name is "", then makes a wrapper with a
    /// single model invisible with respect to DPI scope names.
    explicit Vaggregate_ports(VerilatedContext* contextp, const char* name = "TOP");
    explicit Vaggregate_ports(const char* name = "TOP");
    /// Destroy the model; called (often implicitly) by application code
    virtual ~Vaggregate_ports();
  private:
    VL_UNCOPYABLE(Vaggregate_ports);  ///< Copying not allowed

  public:
    // API METHODS
    /// Evaluate the model.  Application must call when inputs change.
    void eval() { eval_step(); }
    /// Evaluate when calling multiple units/models per time step.
    void eval_step();
    /// Evaluate at end of a timestep for tracing, when using eval_step().
    /// Application must call after all eval() and before time changes.
    void eval_end_step() {}
    /// Simulation complete, run final blocks.  Application must call on completion.
    void final();
    /// Are there scheduled events to handle?
    bool eventsPending();
    /// Returns time at next time slot. Aborts if !eventsPending()
    uint64_t nextTimeSlot();
    /// Trace signals in the model; called by application code
    void trace(VerilatedTraceBaseC* tfp, int levels, int options = 0) { contextp()->trace(tfp, levels, options); }
    /// Retrieve name of this model instance (as passed to constructor).
    const char* name() const;

    // Abstract methods from VerilatedModel
    const char* hierName() const override final;
    const char* modelName() const override final;
    unsigned threads() const override final;
    /// Prepare for cloning the model at the process level (e.g. fork in Linux)
    /// Release necessary resources. Called before cloning.
    void prepareClone() const;
    /// Re-init after cloning the model at the process level (e.g. fork in Linux)
    /// Re-allocate necessary resources. Called after cloning.
    void atClone() const;
  private:
    // Internal functions - trace registration
    void traceBaseModel(VerilatedTraceBaseC* tfp, int levels, int options);
};

#endif  // guard
