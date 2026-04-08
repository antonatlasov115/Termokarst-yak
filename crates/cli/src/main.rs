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

    /// Обработка изображений
    Image {
        #[command(subcommand)]
        command: ImageCommands,
    },

    /// Визуализация результатов симуляции
    Visualize {
        /// Файл с результатами симуляции (JSON)
        #[arg(short, long)]
        input: PathBuf,

        /// Директория для сохранения графиков
        #[arg(short, long, default_value = "visualizations")]
        output_dir: PathBuf,

        /// Тип графика (development, volume, stages, cross-section, all)
        #[arg(short, long, default_value = "all")]
        plot_type: String,
    },
}

#[derive(Subcommand)]
enum ImageCommands {
    /// Загрузить примеры изображений
    Download {
        /// Директория для сохранения
        #[arg(short, long, default_value = "images")]
        output_dir: PathBuf,
    },

    /// Проанализировать изображение
    Analyze {
        /// Файл изображения
        #[arg(short, long)]
        input: PathBuf,

        /// Масштаб (метров на пиксель)
        #[arg(short, long, default_value = "0.1")]
        scale: f64,

        /// Файл для сохранения датасета
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// ID точки наблюдения
        #[arg(long, default_value = "PHOTO")]
        site_id: String,

        /// Координаты (lat,lon)
        #[arg(short, long)]
        coordinates: Option<String>,
    },

    /// Создать синтетическое изображение
    Synthetic {
        /// Файл для сохранения
        #[arg(short, long, default_value = "synthetic.jpg")]
        output: PathBuf,

        /// Диаметр в пикселях
        #[arg(short, long, default_value = "200")]
        diameter: u32,
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

        Commands::Image { command } => match command {
            ImageCommands::Download { output_dir } => image::download(output_dir)?,
            ImageCommands::Analyze {
                input,
                scale,
                output,
                site_id,
                coordinates,
            } => image::analyze(input, scale, output, site_id, coordinates)?,
            ImageCommands::Synthetic { output, diameter } => {
                image::create_synthetic(output, diameter)?
            }
        },

        Commands::Visualize {
            input,
            output_dir,
            plot_type,
        } => visualize::run(input, output_dir, plot_type)?,
    }

    Ok(())
}
