From radium Require Import lang notation.
From refinedrust Require Import typing.

Section lru_model.
    Context {K V : Type} `{EqDecision K}.
    Implicit Types (l : list (K * V)) (k : K) (v : V).

    Fixpoint al_lookup l k : option V :=
        match l with
        | [] => None
        | (k', v') :: l' => if decide (k' = k) then Some v' else al_lookup l' k
    end.

    Fixpoint al_remove l k : list (K * V) :=
        match l with
        | [] => []
        | (k', v') :: l' => if decide (k' = k) then l' else (k', v') :: al_remove l' k
    end.

    Definition al_move_to_front l k : list (K * V) :=
        match al_lookup l k with
        | Some v => (k, v) :: al_remove l k
        | None   => l
    end.

    Definition al_put (cap : nat) l k v : list (K * V) :=
        match al_lookup l k with
        | Some _ => (k, v) :: al_remove l k              (* update + promote *)
        | None   => if bool_decide (length l < cap)
                    then (k, v) :: l                     (* prepend *)
                    else (k, v) :: take (length l - 1) l (* evict + prepend *)
    end.
End lru_model.

Section lru_dll.
    Context `{RRGS : !refinedrustGS Σ}.
    Context {K_rt V_rt : RT}.
    Context (K_ty : type K_rt) (V_ty : type V_rt).
    Context `{!EqDecision (RT_xt K_rt), !Countable (RT_xt K_rt)}.

    Notation lru_map := (gmap K_rt loc).
    Notation lru_elist := (list (K_rt * V_rt)).

    Fixpoint lru_chain π (m : lru_map) (prevp cur tl : loc) (l : lru_elist) : iProp Σ := 
        match l with
        | [] => cur = tl 
        | (k, v) :: l' =>
            ∃ nextp, node_own π cur (Some k) (Some v) prevp nextp
                * m !! k = Some cur
                * lru_chain π m cur nextp tl l'
        end.

    Definition lru_dll π (hd tl : loc) (m : lru_map) (l : lru_elist) : iProp Σ :=
        ∃ firstp lastp,
            node_own π hd None None firstp  (* head.next = first real *)
            * node_own π tl None lastp None (* tail.prev = last real *)
            * lru_chain π m hd firstp tl l 
            * dom m = list_to_set (first <$> l).

End lru_dll.

    
    