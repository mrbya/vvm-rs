// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Primary model header
//
// This header should be included by all source files instantiating the design.
// The class here is then constructed to instantiate the design.
// See the Verilator manual for examples.

#ifndef VERILATED_VPACKED_ARRAY_PORTS_H_
#define VERILATED_VPACKED_ARRAY_PORTS_H_  // guard

#include "verilated.h"

class Vpacked_array_ports__Syms;
class Vpacked_array_ports___024root;

// This class is the main interface to the Verilated model
class alignas(VL_CACHE_LINE_BYTES) Vpacked_array_ports VL_NOT_FINAL : public VerilatedModel {
  private:
    // Symbol table holding complete model state (owned by this class)
    Vpacked_array_ports__Syms* const vlSymsp;

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
    VL_OUT8(&__Vm_sig_first_byte,7,0);
    VL_OUT8(&__Vm_sig_last_byte,7,0);

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
    decltype(__Vm_sig_first_byte) first_byte() {return __Vm_sig_first_byte;}
    void first_byte(decltype(__Vm_sig_first_byte) v) {__Vm_sig_first_byte=v;}
    decltype(__Vm_sig_last_byte) last_byte() {return __Vm_sig_last_byte;}
    void last_byte(decltype(__Vm_sig_last_byte) v) {__Vm_sig_last_byte=v;}

    // CELLS
    // Public to allow access to /* verilator public */ items.
    // Otherwise the application code can consider these internals.

    // Root instance pointer to allow access to model internals,
    // including inlined /* verilator public_flat_* */ items.
    Vpacked_array_ports___024root* const rootp;

    // CONSTRUCTORS
    /// Construct the model; called by application code
    /// If contextp is null, then the model will use the default global context
    /// If name is "", then makes a wrapper with a
    /// single model invisible with respect to DPI scope names.
    explicit Vpacked_array_ports(VerilatedContext* contextp, const char* name = "TOP");
    explicit Vpacked_array_ports(const char* name = "TOP");
    /// Destroy the model; called (often implicitly) by application code
    virtual ~Vpacked_array_ports();
  private:
    VL_UNCOPYABLE(Vpacked_array_ports);  ///< Copying not allowed

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
