import bencher_valid_init from "bencher_valid";
import {
	createEffect,
	createMemo,
	createResource,
	createSignal,
} from "solid-js";

import { createStore } from "solid-js/store";
import type {
	Email,
	JsonAuthToken,
	JsonLogin,
	Jwt,
	PlanLevel,
} from "../../types/bencher";
import { setUser } from "../../util/auth";
import { httpPost } from "../../util/http";
import { NotifyKind, navigateNotify, pageNotify } from "../../util/notify";
import { useSearchParams } from "../../util/url";
import { validEmail, validJwt, validPlanLevel } from "../../util/valid";
import type { FieldHandler } from "../field/Field";
import Field from "../field/Field";
import FieldKind from "../field/kind";
import { AUTH_FIELDS, EMAIL_PARAM, PLAN_PARAM, TOKEN_PARAM } from "./auth";

export interface Props {
	apiUrl: string;
}

const ConfirmForm = (props: Props) => {
	const [bencher_valid] = createResource(
		async () => await bencher_valid_init(),
	);

	const [searchParams, setSearchParams] = useSearchParams();
	const [submitting, setSubmitting] = createSignal(false);
	const [valid, setValid] = createSignal(false);

	const isSendable = (): boolean => {
		return !submitting() && valid();
	};

	const token = createMemo(() => searchParams[TOKEN_PARAM]?.trim() as Jwt);
	const plan = createMemo(() => searchParams[PLAN_PARAM]?.trim() as PlanLevel);
	const email = createMemo(() => searchParams[EMAIL_PARAM]?.trim() as Email);

	const [submitted, setSubmitted] = createSignal();
	const [form, setForm] = createStore<{
		token: {
			value: string;
			valid: null | boolean;
		};
	}>(initForm());

	const [coolDown, setCoolDown] = createSignal(true);
	setTimeout(() => setCoolDown(false), 10000);

	const handleField: FieldHandler = (key, value, valid) => {
		setForm({
			...form,
			[key]: {
				value: value,
				valid: valid,
			},
		});
	};

	const handleSubmit = () => {
		setSubmitting(true);

		const authToken: JsonAuthToken = {
			token: token(),
		};
		httpPost(props.apiUrl, "/v0/auth/confirm", null, authToken)
			.then((resp) => {
				setSubmitting(false);
				const user = resp.data;
				if (setUser(user)) {
					navigateNotify(
						NotifyKind.OK,
						`Hoppy to ${plan() ? "meet you" : "see you again"}, ${
							user.user.name
						}!`,
						"/console",
						[PLAN_PARAM],
						null,
					);
				} else {
					pageNotify(NotifyKind.ERROR, "Invalid user. Please, try again.");
				}
			})
			.catch((error) => {
				setSubmitting(false);
				console.error(error);
				pageNotify(
					NotifyKind.ERROR,
					"Failed to confirm token. Please, try again.",
				);
			});
	};

	const handleResendEmail = () => {
		setSubmitting(true);

		const login: JsonLogin = {
			email: email().trim(),
			plan: plan()?.trim() as PlanLevel,
		};
		httpPost(props.apiUrl, "/v0/auth/login", null, login)
			.then((_resp) => {
				setSubmitting(false);
				navigateNotify(
					NotifyKind.OK,
					`Successful resent email to ${email()}. Please confirm token.`,
					null,
					[PLAN_PARAM, EMAIL_PARAM],
					null,
				);
			})
			.catch((error) => {
				setSubmitting(false);
				console.error(error);
				pageNotify(
					NotifyKind.ERROR,
					`Failed to resend email to ${email()}. Please, try again.`,
				);
			});

		setCoolDown(true);
		setTimeout(() => setCoolDown(false), 30000);
	};

	createEffect(() => {
		if (!bencher_valid()) {
			return;
		}

		const initParams: Record<string, null | string> = {};
		if (!validJwt(searchParams[TOKEN_PARAM])) {
			initParams[TOKEN_PARAM] = null;
		}
		if (!validPlanLevel(searchParams[PLAN_PARAM])) {
			initParams[PLAN_PARAM] = null;
		}
		if (!validEmail(searchParams[EMAIL_PARAM])) {
			initParams[EMAIL_PARAM] = null;
		}
		const token_value = form.token?.value;
		if (validJwt(token_value)) {
			initParams[TOKEN_PARAM] = token_value;
		}
		if (Object.keys(initParams).length !== 0) {
			setSearchParams(initParams);
		}

		const token_valid = form.token?.valid;
		if (typeof token_valid === "boolean" && token_valid !== valid()) {
			setValid(token_valid);
		}

		const jwt = token();
		if (validJwt(jwt) && jwt !== submitted()) {
			setSubmitted(jwt);
			handleSubmit();
		}
	});

	return (
		<>
			<form class="box">
				<Field
					kind={FieldKind.INPUT}
					fieldKey="token"
					value={form?.token?.value}
					valid={form?.token?.valid}
					config={AUTH_FIELDS.token}
					handleField={handleField}
				/>

				<div class="field">
					<p class="control">
						<button
							class="button is-primary is-fullwidth"
							disabled={!isSendable()}
							onClick={(e) => {
								e.preventDefault();
								handleSubmit();
							}}
						>
							Submit
						</button>
					</p>
				</div>
			</form>

			{email() && (
				<>
					<hr />

					<div class="content has-text-centered">
						<button
							class="button is-small is-black is-inverted"
							disabled={submitting() || coolDown()}
							onClick={(e) => {
								e.preventDefault();
								handleResendEmail();
							}}
						>
							<div>Click to resend email to: {email()}</div>
						</button>
					</div>
				</>
			)}
		</>
	);
};

const initForm = () => {
	return {
		token: {
			value: "",
			valid: null,
		},
	};
};

export default ConfirmForm;
