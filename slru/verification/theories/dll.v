From radium Require Import lang notation.
From refinedrust Require Import typing shims.

Section slru_dll.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}. 
  Context `{!EqDecision K_rt}.

  Notation node_rt := ((option (K_rt * V_rt) * loc * loc)%type).

  Context (node_ty : type node_rt).

  Fixpoint dll (π : thread_id) (xs : list (K_rt * V_rt)) (cur next tail tprev: loc) : iProp Σ :=
    match xs with
    | [] => ⌜next = tail⌝ ∗ ⌜tprev = cur⌝
    | (nextk, nextv) :: xs' =>
        ∃ nextnext : loc,
          cur◁ₗ[π, Owned] #(-[ #(Some -[nextk; nextv]); #nextnext; #cur]) @ (◁ node_ty) ∗
          dll π xs' next nextnext tail tprev
    end.
End slru_dll.