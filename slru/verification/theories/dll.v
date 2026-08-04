From radium Require Import lang notation.
From refinedrust Require Import typing shims.

Section slru_dll.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.

  (* Ownership of a single node, ListNode field order: key, val, prev, next. *)
  Context (node_own :
    thread_id →
    loc →
    option (place_rfn K_rt) →
    option (place_rfn V_rt) →
    loc →
    loc →
    iProp Σ
  ).

  Notation elist := (list (RT_xt K_rt * RT_xt V_rt)).

  Fixpoint slru_chain (π : thread_id) (prevp cur tl lastp : loc) (l : elist) : iProp Σ :=
    match l with
    | [] => ⌜cur = tl⌝ ∗ ⌜prevp = lastp⌝
    | (k, v) :: l' =>
        ∃ nextp : loc,
          node_own π cur (Some (#($# k))) (Some (#($# v))) prevp nextp ∗
          slru_chain π cur nextp tl lastp l'
    end.

  Definition slru_dll (π : thread_id) (hd tl : loc) (l : elist) : iProp Σ :=
    ∃ firstp lastp : loc,
      node_own π hd None None NULL_loc firstp ∗
      node_own π tl None None lastp NULL_loc ∗
      slru_chain π hd firstp tl lastp l.
End slru_dll.