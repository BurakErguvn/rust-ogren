// app/API.md ile birebir uyumlu tipler.

export interface ExerciseProgress {
  completed: boolean;
  score: number | null;
  failed_submits: number;
}

export interface TocPage {
  id: string;
  title: string;
  translated: boolean;
  exercise: ExerciseProgress | null;
}

export interface TocChapter {
  id: string;
  title: string;
  pages: TocPage[];
}

export interface Toc {
  chapters: TocChapter[];
  progress: { completed: number; total: number; score_sum: number };
}

export interface PageData {
  id: string;
  title: string;
  translated: boolean;
  markdown: string;
  prev: string | null;
  next: string | null;
  exercise: ExerciseProgress | null;
  /** Uygulamaya özel notlar; alan yoksa boş kabul edilir. */
  notes?: PageNote[];
}

export interface PageNote {
  /** Notun hemen önüne konacağı başlığın id'si; null: sayfanın başı (H1'den sonra). */
  before: string | null;
  markdown: string;
}

export interface ExerciseFile {
  path: string;
  content: string;
  editable: boolean;
  /** Varsayılan `rust`. `toml` dosyaları (Cargo.toml) LSP'ye gönderilmez. */
  language?: 'rust' | 'toml';
  /** Sunucu belirtirse: öğrencinin oluşturduğu, silinebilir dosya. */
  deletable?: boolean;
  /** Bu dosyada gizli test kodu var (içerikten çıkarıldı). */
  has_hidden_tests?: boolean;
}

export interface ExerciseFiles {
  id: string;
  workspace_root_uri: string;
  exercise_root_uri: string;
  files: ExerciseFile[];
  /** `tests/` altındaki, hiç listelenmeyen gizli test dosyalarının sayısı. */
  hidden_test_files?: number;
  progress: ExerciseProgress;
}

export type RunStatus = 'passed' | 'failed' | 'compile_error' | 'timeout';

export interface TestCaseResult {
  name: string;
  passed: boolean;
  output: string;
}

export interface CompilerDiagnostic {
  file: string;
  /** Gizli test kodundaki teşhislerde `null`. */
  line: number | null;
  column: number | null;
  end_line: number | null;
  end_column: number | null;
  severity: 'error' | 'warning' | string;
  message: string;
  rendered?: string;
  /** Teşhis gizli test kodunda: konum yok, koda gidilemez. */
  hidden?: boolean;
}

export interface RunResult {
  kind: 'test' | 'submit';
  status: RunStatus;
  tests: TestCaseResult[];
  tests_passed: number;
  tests_total: number;
  diagnostics: CompilerDiagnostic[];
  log: string;
  duration_ms: number;
  progress: ExerciseProgress;
}
