From radium Require Import lang notation.
From refinedrust Require Import typing shims.

Section lru_dll.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.
  Context `{!EqDecision (RT_xt K_rt), !Countable (RT_xt K_rt)}.
  (* opaque here; the concrete instance is plugged in at the struct invariant *)
  Context (node_own : 
                thread_id → 
                loc →
                option (place_rfn K_rt) → 
                option (place_rfn V_rt) →
                loc → 
                loc → 
                iProp Σ
            ).

  Notation lru_map   := (gmap (RT_xt K_rt) loc).
  Notation lru_elist := (list (RT_xt K_rt * RT_xt V_rt)).

  Fixpoint lru_chain (π : thread_id) (m : lru_map) (prevp cur tl : loc)
                     (l : lru_elist) : iProp Σ :=
    match l with
    | [] => ⌜cur = tl⌝
    | (k, v) :: l' =>
        ∃ nextp : loc,
          node_own π cur (Some (#($# k))) (Some (#($# v))) prevp nextp ∗
          ⌜m !! k = Some cur⌝ ∗
          lru_chain π m cur nextp tl l'
    end.

  Definition lru_dll (π : thread_id) (hd tl : loc)
                     (m : lru_map) (l : lru_elist) : iProp Σ :=
    ∃ hdprev firstp lastp tlnext : loc,
      node_own π hd None None hdprev firstp ∗ 
      node_own π tl None None lastp  tlnext  ∗
      lru_chain π m hd firstp tl l ∗
      ⌜dom m = list_to_set (fst <$> l)⌝.
End lru_dll.