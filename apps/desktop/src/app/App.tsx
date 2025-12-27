import { BrowserRouter, Route, Routes } from "react-router-dom";
import { MixStartPage } from "../pages/MixStart/MixStartPage";
import { PreviewPage } from "../pages/Preview/PreviewPage";

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<MixStartPage />} />
        <Route path="/preview/:jobId" element={<PreviewPage />} />
      </Routes>
    </BrowserRouter>
  );
}
