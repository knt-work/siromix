import { BrowserRouter, Route, Routes } from "react-router-dom";
import { MixStartPage } from "../pages/MixStart/MixStartPage";
import { PreviewPage } from "../pages/Preview/PreviewPage";
import { MixedResultPage } from "../pages/MixedResult/MixedResultPage";

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<MixStartPage />} />
        <Route path="/preview/:jobId" element={<PreviewPage />} />
        <Route path="/result" element={<MixedResultPage />} />
      </Routes>
    </BrowserRouter>
  );
}
