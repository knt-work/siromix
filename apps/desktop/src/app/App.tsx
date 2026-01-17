import { BrowserRouter, Route, Routes } from "react-router-dom";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { MixStartPage } from "../pages/MixStart/MixStartPage";
import { PreviewPage } from "../pages/Preview/PreviewPage";
import { MixedResultPage } from "../pages/MixedResult/MixedResultPage";

export function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<MixStartPage />} />
          <Route path="/preview/:jobId" element={<PreviewPage />} />
          <Route path="/result" element={<MixedResultPage />} />
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
}
