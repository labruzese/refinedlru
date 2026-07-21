From radium Require Import lang notation.
From refinedrust Require Import typing shims.

Section lru_model.
    Context {K V : Type} `{EqDecision K}.
    (* l is our cache type, we're generic over K, V*)
    Implicit Types (l : list (K * V)) (k : K) (v : V).

    (* cache lookup *)
    Fixpoint l_lookup l k : option V :=
        match l with
        | [] => None
        | (k', v') :: l' => if decide (k' = k) then Some v' else l_lookup l' k
    end.

    (* cache remove *)
    Fixpoint l_remove l k : list (K * V) :=
        match l with
        | [] => []
        | (k', v') :: l' => if decide (k' = k) then l' else (k', v') :: l_remove l' k
    end.

    (* change priority of key to first *)
    Definition l_move_to_front l k : list (K * V) :=
        match l_lookup l k with
        | Some v => (k, v) :: l_remove l k
        | None   => l
    end.

    (* cache put *)
    Definition l_put (cap : nat) l k v : list (K * V) :=
        match l_lookup l k with
        | Some _ => (k, v) :: l_remove l k              (* update + promote *)
        | None   => if bool_decide (length l < cap)
                    then (k, v) :: l                     (* prepend *)
                    else (k, v) :: take (length l - 1) l (* evict + prepend *)
    end.
End lru_model.