import "./styles/styles.scss";

import {
  createSignal,
  createEffect,
  lazy,
  Component,
  createMemo,
  For,
  Switch,
  Match,
} from "solid-js";
import {
  Routes,
  Route,
  Navigate,
  useLocation,
  useNavigate,
} from "solid-app-router";

import { Navbar } from "./components/site/navbar/Navbar";
import { site_analytics } from "./components/site/site_analytics";
import SiteFooter from "./components/site/pages/SiteFooter";
import { projectSlug } from "./components/console/ConsolePage";
import { BENCHER_TITLE } from "./components/site/pages/LandingPage";
import { BENCHER_USER_KEY, NotifyKind } from "./components/site/util";
import validator from "validator";

const AuthRoutes = lazy(() => import("./components/auth/AuthRoutes"));
const LandingPage = lazy(() => import("./components/site/pages/LandingPage"));
const ConsoleRoutes = lazy(() => import("./components/console/ConsoleRoutes"));
const DocsRoutes = lazy(() => import("./components/docs/DocsRoutes"));
const LegalRoutes = lazy(() => import("./components/legal/LegalRoutes"));
const Repo = lazy(() => import("./components/site/Repo"));
const Notification = lazy(() => import("./components/site/Notification"));

const initUser = () => {
  return {
    user: {
      uuid: null,
      name: null,
      slug: null,
      email: null,
      admin: null,
      locked: null,
    },
    token: null,
  };
};

const App: Component = () => {
  const [title, setTitle] = createSignal<string>(BENCHER_TITLE);
  const [user, setUser] = createSignal(initUser());

  const location = useLocation();
  const pathname = createMemo(() => location.pathname);

  const [organization_slug, setOrganizationSlug] = createSignal<null | String>(
    null
  );
  // The project slug can't be a resource because it isn't 100% tied to the URL
  const [project_slug, setProjectSlug] = createSignal<String>(
    projectSlug(pathname)
  );

  createEffect(() => {
    if (document.title !== title()) {
      document.title = title();
    }
  });

  const handleUser = (user) => {
    window.localStorage.setItem(BENCHER_USER_KEY, JSON.stringify(user));
    setUser(user);
  };

  const removeUser = (user) => {
    window.localStorage.clear();
    setUser(initUser());
  };

  const handleNotification = (kind: NotifyKind, text: string) => {
    console.log({ kind: kind, text: text });
    // console.log(notification());
    // setNotification({ kind: kind, text: text });
    // console.log(notification());
    // setTimeout(() => {
    //   removeNotification();
    // }, 4000);
  };

  setInterval(() => {
    if (user()?.token === null) {
      const user = JSON.parse(window.localStorage.getItem(BENCHER_USER_KEY));
      // TODO properly validate entire user
      if (user?.token && validator.isJWT(user.token)) {
        setUser(user);
      }
    }
  }, 1000);

  const analytics = createMemo(site_analytics);
  const handleTitle = (new_title) => {
    const bencher_title = `${new_title} - Bencher`;
    if (title() !== bencher_title) {
      setTitle(bencher_title);
      analytics()?.page();
    }
  };

  // const getNotification = () => {
  //   console.log("GETTING NOTIFICATION");
  //   let color: string;
  //   switch (notification().kind) {
  //     case NotificationKind.OK:
  //       color = "is-success";
  //       break;
  //     case NotificationKind.ALERT:
  //       color = "is-primary";
  //       break;
  //     case NotificationKind.ERROR:
  //       color = "is-danger";
  //       break;
  //     default:
  //       color = "";
  //   }
  //   return (
  //     <div class={`notification ${color}`}>
  //       🐰 {notification().text}
  //       <button
  //         class="delete"
  //         onClick={(e) => {
  //           e.preventDefault();
  //           removeNotification();
  //         }}
  //       />
  //     </div>
  //   );
  // };

  return (
    <>
      <Navbar
        user={user}
        organization_slug={organization_slug}
        project_slug={project_slug}
        handleProjectSlug={setProjectSlug}
      />

      <Notification />

      {/* <div>
        <Switch fallback={<></>}>
          <Match when={typeof notification().text === "string"}>
            <section class="section">
              <div class="container">{getNotification()}</div>
            </section>
          </Match>
        </Switch>
      </div> */}

      {/* <div>
        <Switch fallback={<></>}>
          <Match when={redirect() && redirect() != pathname()}>
            <Navigate href={getRedirect()} />
          </Match>
        </Switch>
      </div> */}

      <Routes>
        <Route
          path="/"
          element={
            <LandingPage
              analytics={analytics}
              user={user}
              handleTitle={setTitle}
            />
          }
        />

        {/* Auth Routes */}
        <Route path="/auth">
          <AuthRoutes
            handleTitle={handleTitle}
            user={user}
            handleUser={handleUser}
            removeUser={removeUser}
            handleNotification={handleNotification}
          />
        </Route>

        {/* Console Routes */}
        <Route path="/console">
          <ConsoleRoutes
            user={user}
            pathname={pathname}
            organization_slug={organization_slug}
            project_slug={project_slug}
            handleTitle={handleTitle}
            handleOrganizationSlug={setOrganizationSlug}
            handleProjectSlug={setProjectSlug}
          />
        </Route>

        {/* Docs Routes */}
        <Route path="/docs">
          <DocsRoutes />
        </Route>

        {/* Auth Routes */}
        <Route path="/legal">
          <LegalRoutes handleTitle={handleTitle} />
        </Route>

        {/* GitHub repo shortcut */}
        <Route path="/repo" element={<Repo />} />
      </Routes>

      <For each={[...Array(16).keys()]}>{(_k, _i) => <br />}</For>
      <SiteFooter />
    </>
  );
};

export default App;
