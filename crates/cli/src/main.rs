//! CLI для симуляции термокарстовых образований в Якутии

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

use commands::*;

#[derive(Parser)]
#[command(name = "thermokarst")]
#[command(about = "Симуляция термокарстовых образований в Якутии", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Запустить симуляцию
    Simulate {
        /// Регион Якутии (north, central, south)
        #[arg(short, long, default_value = "central")]
        region: String,

        /// Количество лет симуляции
        #[arg(short, long, default_value = "50")]
        years: u32,

        /// Файл для сохранения результатов (JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Показать детальный вывод
        #[arg(short, long)]
        verbose: bool,
    },

    /// Запустить батч-симуляцию для всех регионов
    Batch {
        /// Количество лет симуляции
        #[arg(short, long, default_value = "50")]
        years: u32,

        /// Директория для сохранения результатов
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Использовать параллельное выполнение
        #[arg(short, long)]
        parallel: bool,
    },

    /// Анализ стабильности существующих данных
    Analyze {
        /// Файл с данными симуляции (JSON)
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Создать пример конфигурации
    Config {
        /// Файл для сохранения конфигурации
        #[arg(short, long, default_value = "config.json")]
        output: PathBuf,
    },

    /// Работа с датасетами наблюдений
    Dataset {
        #[command(subcommand)]
        command: DatasetCommands,
    },
}

#[derive(Subcommand)]
enum DatasetCommands {
    /// Создать пример датасета
    Create {
        /// Файл для сохранения
        #[arg(short, long, default_value = "dataset.json")]
        output: PathBuf,
    },

    /// Показать информацию о датасете
    Info {
        /// Файл датасета
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Калибровать модель по датасету
    Calibrate {
        /// Файл датасета
        #[arg(short, long)]
        input: PathBuf,

        /// Файл для сохранения параметров
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Simulate {
            region,
            years,
            output,
            verbose,
        } => simulate::run(region, years, output, verbose)?,

        Commands::Batch {
            years,
            output_dir,
            parallel,
        } => batch::run(years, output_dir, parallel)?,

        Commands::Analyze { input } => analyze::run(input)?,

        Commands::Config { output } => config::run(output)?,

        Commands::Dataset { command } => match command {
            DatasetCommands::Create { output } => dataset::create(output)?,
            DatasetCommands::Info { input } => dataset::info(input)?,
            DatasetCommands::Calibrate { input, output } => dataset::calibrate(input, output)?,
        },
    }

    Ok(())
}
