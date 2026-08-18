From stdpp Require Import base list ssreflect.

(* Functional model of the cache. *)

(* Generic list helpers *)

Lemma fmap_delete {A B} (f : A → B) (xs : list A) (i : nat) :
  f <$> delete i xs = delete i (f <$> xs).
Proof. by rewrite !delete_take_drop fmap_app fmap_take fmap_drop. Qed.

Lemma removelast_delete {A} (xs : list A) :
  removelast xs = delete (pred (length xs)) xs.
Proof.
  induction xs as [|x xs IH]; first done.
  destruct xs as [|y xs]; first done. simpl in *. by rewrite IH.
Qed.

Lemma NoDup_delete {A} (xs : list A) (i : nat) : NoDup xs → NoDup (delete i xs).
Proof. intros H. by apply (sublist_NoDup _ _ H (sublist_delete xs i)). Qed.

Lemma elem_of_delete {A} (xs : list A) (i : nat) (y : A) :
  y ∈ delete i xs → y ∈ xs.
Proof. intros H. by apply (elem_of_sublist _ _ _ H (sublist_delete xs i)). Qed.

(* [i] held the only occurrence of [y], so deleting it removes [y] entirely. *)
Lemma NoDup_not_elem_of_delete {A} (xs : list A) (i : nat) y :
  NoDup xs → xs !! i = Some y → y ∉ delete i xs.
Proof.
  intros Hnd Hi Hin.
  apply lookup_lt_Some in Hi as Hlt.
  apply list_elem_of_lookup_1 in Hin as [j Hj].
  rewrite delete_take_drop in Hj.
  destruct (decide (j < i)) as [Hlt'|Hge].
  - rewrite lookup_app_l in Hj; last (rewrite length_take_le; lia).
    rewrite lookup_take_lt in Hj; last lia.
    assert (j = i) by (eapply NoDup_lookup; done). lia.
  - rewrite lookup_app_r in Hj; last (rewrite length_take_le; lia).
    rewrite length_take_le in Hj; last lia. rewrite lookup_drop in Hj.
    assert (S i + (j - i) = i) by (eapply NoDup_lookup; done). lia.
Qed.

Section lru_model.
  Context {K V : Type} `{!EqDecision K}.
  Implicit Types (l : list (K * V)) (k : K) (v : V) (i : nat).

  (* operations *)

  (* lookup: the first entry with key [k]. *)
  Fixpoint l_lookup l k : option V :=
    match l with
    | [] => None
    | (k', v') :: l' => if decide (k' = k) then Some v' else l_lookup l' k
    end.

  (* drop the first entry with key [k]. *)
  Fixpoint l_remove l k : list (K * V) :=
    match l with
    | [] => []
    | (k', v') :: l' => if decide (k' = k) then l' else (k', v') :: l_remove l' k
    end.

  (* promote [k] to MRU: what [get] does to the cache. *)
  Definition l_move_to_front l k : list (K * V) :=
    match l_lookup l k with
    | Some v => (k, v) :: l_remove l k
    | None => l
    end.

  Definition l_put (cap : nat) l k v : list (K * V) :=
    match l_lookup l k with
    | Some _ => (k, v) :: l_remove l k          (* update + promote *)
    | None => if bool_decide (length l < cap)
              then (k, v) :: l                  (* prepend *)
              else (k, v) :: removelast l       (* evict LRU + prepend *)
    end.

  (* [l_lookup] against list indexing *)

  Lemma l_lookup_None l k : l_lookup l k = None ↔ k ∉ l.*1.
  Proof.
    induction l as [|[k' v'] l IH]; simpl.
    { split; intros _; [set_solver | done]. }
    case_decide as Hk; simplify_eq/=.
    - split; first done. rewrite elem_of_cons. naive_solver.
    - rewrite IH elem_of_cons. naive_solver.
  Qed.

  Lemma l_lookup_Some_lookup l k v :
    l_lookup l k = Some v → ∃ i, l !! i = Some (k, v).
  Proof.
    induction l as [|[k' v'] l IH]; simpl; first done.
    case_decide as Hk; simplify_eq/=.
    - intros [= ->]. by exists 0%nat.
    - intros [i Hi]%IH. by exists (S i).
  Qed.

  Lemma lookup_l_lookup l i k v :
    NoDup l.*1 → l !! i = Some (k, v) → l_lookup l k = Some v.
  Proof.
    revert i. induction l as [|[k' v'] l IH]; intros [|i] Hnd Hi; simplify_eq/=.
    { by rewrite decide_True. }
    apply NoDup_cons in Hnd as [Hnotin Hnd].
    case_decide as Hk; last by apply (IH i).
    (* [k] occurs at index [S i], so it cannot also head the list *)
    subst k'. exfalso. apply Hnotin, list_elem_of_fmap.
    exists (k, v). split; first done. by eapply list_elem_of_lookup_2.
  Qed.

  (* [l_remove] against [delete] *)

  Lemma l_remove_id l k : k ∉ l.*1 → l_remove l k = l.
  Proof.
    induction l as [|[k' v'] l IH]; simpl; first done.
    rewrite elem_of_cons. intros Hnotin.
    case_decide; first naive_solver. f_equal. apply IH. naive_solver.
  Qed.

  Lemma l_remove_delete l i k v :
    NoDup l.*1 → l !! i = Some (k, v) → l_remove l k = delete i l.
  Proof.
    revert i. induction l as [|[k' v'] l IH]; intros [|i] Hnd Hi; simplify_eq/=.
    { by rewrite decide_True. }
    apply NoDup_cons in Hnd as [Hnotin Hnd].
    case_decide as Hk.
    - subst k'. exfalso. apply Hnotin, list_elem_of_fmap.
      exists (k, v). split; first done. by eapply list_elem_of_lookup_2.
    - f_equal. by apply (IH i).
  Qed.

  (* [get]: promote the hit entry *)

  Lemma l_move_to_front_hit l i k v :
    NoDup l.*1 → l !! i = Some (k, v) →
    l_move_to_front l k = (k, v) :: delete i l.
  Proof.
    intros Hnd Hi. rewrite /l_move_to_front (lookup_l_lookup _ _ _ _ Hnd Hi).
    f_equal. by eapply l_remove_delete.
  Qed.

  Lemma l_move_to_front_miss l k : k ∉ l.*1 → l_move_to_front l k = l.
  Proof.
    intros Hnotin. rewrite /l_move_to_front. by apply l_lookup_None in Hnotin as ->.
  Qed.

  (* [put], one equation per branch of the Rust code *)

  Lemma l_put_hit cap l i k v w :
    NoDup l.*1 → l !! i = Some (k, w) →
    l_put cap l k v = (k, v) :: delete i l.
  Proof.
    intros Hnd Hi. rewrite /l_put (lookup_l_lookup _ _ _ _ Hnd Hi).
    f_equal. by eapply l_remove_delete.
  Qed.

  Lemma l_put_miss_room cap l k v :
    k ∉ l.*1 → length l < cap → l_put cap l k v = (k, v) :: l.
  Proof.
    intros Hnotin Hcap. rewrite /l_put.
    apply l_lookup_None in Hnotin as ->. by rewrite bool_decide_true.
  Qed.

  Lemma l_put_miss_full cap l k v :
    k ∉ l.*1 → cap ≤ length l → l_put cap l k v = (k, v) :: removelast l.
  Proof.
    intros Hnotin Hcap. rewrite /l_put.
    apply l_lookup_None in Hnotin as ->. rewrite bool_decide_false //. lia.
  Qed.

  (* Key uniqueness is preserved *)

  Lemma NoDup_fst_delete l i : NoDup l.*1 → NoDup (delete i l).*1.
  Proof. intros ?. rewrite fmap_delete. by apply NoDup_delete. Qed.

  Lemma not_elem_of_fst_delete l i k : k ∉ l.*1 → k ∉ (delete i l).*1.
  Proof.
    intros Hnotin Hin. apply Hnotin. rewrite fmap_delete in Hin.
    by eapply elem_of_delete.
  Qed.

  Lemma not_elem_of_fst_delete_at l i k v :
    NoDup l.*1 → l !! i = Some (k, v) → k ∉ (delete i l).*1.
  Proof.
    intros Hnd Hi. rewrite fmap_delete.
    eapply NoDup_not_elem_of_delete; first done.
    by rewrite list_lookup_fmap Hi.
  Qed.

  Lemma NoDup_l_put cap l k v : NoDup l.*1 → NoDup (l_put cap l k v).*1.
  Proof.
    intros Hnd. rewrite /l_put.
    destruct (l_lookup l k) as [w|] eqn:Hlk.
    - apply l_lookup_Some_lookup in Hlk as [i Hi].
      rewrite (l_remove_delete _ _ _ _ Hnd Hi) /=.
      apply NoDup_cons. split; last by apply NoDup_fst_delete.
      by eapply not_elem_of_fst_delete_at.
    - apply l_lookup_None in Hlk.
      case_bool_decide; simpl; apply NoDup_cons; split; try done.
      + rewrite removelast_delete. by apply not_elem_of_fst_delete.
      + rewrite removelast_delete. by apply NoDup_fst_delete.
  Qed.

End lru_model.