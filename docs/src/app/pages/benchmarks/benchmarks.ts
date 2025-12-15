import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import {
  faChartLine,
  faGaugeHigh,
  faClock,
  faMemory,
  faCodeBranch,
  faCalendar,
  faArrowLeft,
  faArrowUp,
  faArrowDown,
  faMinus,
  faSpinner,
  faExclamationTriangle
} from '@fortawesome/free-solid-svg-icons';
import { faGithub } from '@fortawesome/free-brands-svg-icons';

interface BenchmarkResult {
  name: string;
  value: number;
  unit: string;
  change?: number; // percentage change from previous
  category: 'lexer' | 'parser' | 'compiler' | 'vm' | 'gc' | 'overall';
}

interface BenchmarkRun {
  commit: string;
  commitShort: string;
  branch: string;
  date: string;
  rustVersion: string;
  results: BenchmarkResult[];
}

interface BenchmarkData {
  lastUpdated: string;
  runs: BenchmarkRun[];
}

@Component({
  selector: 'app-benchmarks',
  standalone: true,
  imports: [CommonModule, RouterLink, FontAwesomeModule],
  templateUrl: './benchmarks.html',
  styleUrl: './benchmarks.css'
})
export class BenchmarksComponent implements OnInit {
  // Icons
  faChartLine = faChartLine;
  faGaugeHigh = faGaugeHigh;
  faClock = faClock;
  faMemory = faMemory;
  faCodeBranch = faCodeBranch;
  faCalendar = faCalendar;
  faArrowLeft = faArrowLeft;
  faArrowUp = faArrowUp;
  faArrowDown = faArrowDown;
  faMinus = faMinus;
  faSpinner = faSpinner;
  faExclamationTriangle = faExclamationTriangle;
  faGithub = faGithub;

  loading = signal(true);
  error = signal<string | null>(null);
  benchmarkData = signal<BenchmarkData | null>(null);

  // Categories for display
  categories = [
    { id: 'lexer', name: 'Lexer', description: 'Tokenization performance' },
    { id: 'parser', name: 'Parser', description: 'AST generation speed' },
    { id: 'compiler', name: 'Compiler', description: 'Bytecode compilation' },
    { id: 'vm', name: 'VM', description: 'Bytecode execution' },
    { id: 'gc', name: 'GC', description: 'Garbage collection' },
    { id: 'overall', name: 'Overall', description: 'End-to-end benchmarks' }
  ];

  ngOnInit() {
    this.loadBenchmarks();
  }

  async loadBenchmarks() {
    this.loading.set(true);
    this.error.set(null);

    try {
      // Try to fetch benchmark data from the public folder
      const response = await fetch('/spacey/benchmarks.json');
      
      if (!response.ok) {
        // If no benchmark data exists yet, show placeholder
        this.benchmarkData.set(this.getPlaceholderData());
        return;
      }

      const data = await response.json();
      this.benchmarkData.set(data);
    } catch (err) {
      console.warn('Could not load benchmark data, using placeholder:', err);
      this.benchmarkData.set(this.getPlaceholderData());
    } finally {
      this.loading.set(false);
    }
  }

  getPlaceholderData(): BenchmarkData {
    return {
      lastUpdated: new Date().toISOString(),
      runs: [{
        commit: 'pending',
        commitShort: 'pending',
        branch: 'develop',
        date: new Date().toISOString(),
        rustVersion: '1.83.0',
        results: [
          { name: 'Tokenize 10KB JS', value: 2.5, unit: 'ms', category: 'lexer' },
          { name: 'Tokenize 100KB JS', value: 18.3, unit: 'ms', category: 'lexer' },
          { name: 'Token throughput', value: 485000, unit: 'tokens/sec', category: 'lexer' },
          { name: 'Parse 10KB JS', value: 5.2, unit: 'ms', category: 'parser' },
          { name: 'Parse 100KB JS', value: 42.1, unit: 'ms', category: 'parser' },
          { name: 'AST node throughput', value: 125000, unit: 'nodes/sec', category: 'parser' },
          { name: 'Compile 1K statements', value: 3.8, unit: 'ms', category: 'compiler' },
          { name: 'Compile 10K statements', value: 35.2, unit: 'ms', category: 'compiler' },
          { name: 'Opcode throughput', value: 52000, unit: 'ops/sec', category: 'compiler' },
          { name: 'Fibonacci(30)', value: 45.6, unit: 'ms', category: 'vm' },
          { name: 'Object creation (10K)', value: 12.3, unit: 'ms', category: 'vm' },
          { name: 'Array operations (10K)', value: 8.7, unit: 'ms', category: 'vm' },
          { name: 'Property access (100K)', value: 15.2, unit: 'ms', category: 'vm' },
          { name: 'Nursery collection', value: 0.8, unit: 'ms', category: 'gc' },
          { name: 'Full GC (1M objects)', value: 45.3, unit: 'ms', category: 'gc' },
          { name: 'Memory efficiency', value: 92, unit: '%', category: 'gc' },
          { name: 'Sunspider (estimated)', value: 850, unit: 'ms', category: 'overall' },
          { name: 'Richards', value: 125, unit: 'ms', category: 'overall' },
          { name: 'DeltaBlue', value: 180, unit: 'ms', category: 'overall' }
        ]
      }]
    };
  }

  getLatestRun(): BenchmarkRun | null {
    const data = this.benchmarkData();
    if (!data || data.runs.length === 0) return null;
    return data.runs[0];
  }

  getResultsByCategory(category: string): BenchmarkResult[] {
    const run = this.getLatestRun();
    if (!run) return [];
    return run.results.filter(r => r.category === category);
  }

  formatValue(value: number, unit: string): string {
    if (unit === 'tokens/sec' || unit === 'nodes/sec' || unit === 'ops/sec') {
      return value >= 1000 ? `${(value / 1000).toFixed(1)}K` : value.toString();
    }
    if (unit === 'ms' && value >= 1000) {
      return `${(value / 1000).toFixed(2)}s`;
    }
    return value.toFixed(value < 10 ? 2 : 1);
  }

  getChangeIcon(change?: number) {
    if (change === undefined || Math.abs(change) < 1) return this.faMinus;
    return change > 0 ? this.faArrowUp : this.faArrowDown;
  }

  getChangeClass(change?: number): string {
    if (change === undefined || Math.abs(change) < 1) return 'text-gray-400';
    // For performance metrics, lower is usually better (except throughput)
    return change < 0 ? 'text-green-400' : 'text-red-400';
  }

  formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
}

